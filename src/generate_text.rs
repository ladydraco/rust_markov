
use std::cmp;
use std::collections::HashMap;
use rand;
use rand::random;
use num;
use num::traits::NumCast;
use gather_stats::{
	Stats,
	OrderStats,
	CharChoiceStats,
	};
use sentence_watcher::SentenceWatcher;
use paragraph_watcher::ParagraphWatcher;

#[derive(Debug)]
pub struct Args {
	pub input_filename: String,
	pub output_filename: String,
	pub lower_order_bound: usize,
	pub higher_order_bound: usize,
	pub max_tries: usize,
	pub distortion_factor: i32,
	pub output_amount: usize,
	pub use_html: bool
}

#[derive(Copy, Clone)]
pub enum TextEvent {
	CharGenerated,
	OutputComplete,
	SentenceComplete(i32), // Contains word count of completed sentence.
	ParagraphComplete(i32), // Contains sentence count of completed paragraph.
}

pub struct Generator<'a> {
	// immutables:
	stats: &'a Stats<'a>,
	max_order: usize,
	min_order: usize,
	output_amount: usize,
	use_html: bool,
	distortion_factor: i32,

	// output:
	output_buffer: String,

	// current state:

	// char-level:
	current: String,
	current_order: usize,
	total: usize,
	change_order_counter: i32,
	distortions: CharChoiceStats,

	// sentence-level:
	sentence_watcher: SentenceWatcher,
	pub current_sentence_length: i32,

	// paragraph-level:
	paragraph_watcher: ParagraphWatcher,
	pub current_paragraph_length: i32,
}

impl<'a> Generator<'a> {
	pub fn new(args: &Args, stats: &'a Stats<'a>) -> Generator<'a> {
		let mut output_buffer = String::new();

		if args.use_html {
			output_buffer.push_str("<meta charset=\"UTF-8\">");
			output_buffer.push_str("<style type=\"text/css\"> body { white-space: pre-wrap; } ");
			for i in args.lower_order_bound..args.higher_order_bound + 1 {
				
				let a = args.higher_order_bound + 1 - args.lower_order_bound;
				let b = i - args.lower_order_bound;
				let c = a - b - 1;
				let multiplier = c as f64 / a as f64;

				let value = (multiplier * 248.0) as i32;
				output_buffer.push_str(".order-");
				output_buffer.push_str(&i.to_string());
				output_buffer.push_str("{ ");

				output_buffer.push_str(" color: rgb(");
				output_buffer.push_str(&value.to_string());
				output_buffer.push_str(",");
				output_buffer.push_str(&value.to_string());
				output_buffer.push_str(",");
				output_buffer.push_str(&value.to_string());
				output_buffer.push_str(");\n");

				output_buffer.push_str("}\n");
			}
			output_buffer.push_str("</style>");
		}

		let generator = Generator {
			stats: stats,
			max_order: args.higher_order_bound,
			min_order: args.lower_order_bound,
			output_amount: args.output_amount,
			use_html: args.use_html,
			distortion_factor: args.distortion_factor,

			output_buffer: output_buffer,

			current: String::new(),
			current_order: args.higher_order_bound,
			total: 0,
			change_order_counter: 0,
			distortions: CharChoiceStats {
				total_usages: 0,
				options: HashMap::new()
			},
			sentence_watcher: SentenceWatcher::new(),
			current_sentence_length: 0,
			paragraph_watcher: ParagraphWatcher::new(),
			current_paragraph_length: 0,
		};

		return generator;
	}

	pub fn sync(&mut self, target: &Generator) {
		self.output_buffer.clear();
		self.current.clear();

		self.output_buffer.push_str(&target.output_buffer);
		self.current.push_str(&target.current);

		self.current_order = target.current_order;
		self.total = target.total;
		self.change_order_counter = target.change_order_counter;
		self.current_sentence_length = target.current_sentence_length;
		self.current_paragraph_length = target.current_paragraph_length;

		self.sentence_watcher.sync(&target.sentence_watcher);
		self.paragraph_watcher.sync(&target.paragraph_watcher);
	}

	// Choose random starting string (encountered in the input text) 
	//  of length MAX_ORDER.

	pub fn start(&mut self) {
		let sentence_starts = &self.stats.sentence_stats.starts;
		let mut choice_index = pick_random_in_range(0, sentence_starts.len() - 1);
		while sentence_starts[choice_index].chars().next().unwrap().is_lowercase() {
			choice_index = pick_random_in_range(0, sentence_starts.len() - 1);
		}

		for (i, next_char) in sentence_starts[choice_index].chars().enumerate() {
			if i < self.current_order {
				self.current.push(next_char);
				self.total += 1;
				self.sentence_watcher.watch(next_char);
			}
		}

		let start_output = self.current.clone();
		self.output(&start_output);

		let sentence_choice_index = pick_random_in_range(0, self.stats.sentence_stats.stats_for_state.len() - 1);
		self.current_sentence_length = *self.stats.sentence_stats.stats_for_state.keys().nth(sentence_choice_index).unwrap();

		let paragraph_choice_index = pick_random_in_range(0, self.stats.paragraph_stats.stats_for_state.len() - 1);
		self.current_paragraph_length = *self.stats.paragraph_stats.stats_for_state.keys().nth(paragraph_choice_index).unwrap();
	}

	// Generate characters that follow the starting string chosen by
	//  following random paths through the generated statistics.

	#[allow(dead_code)]
	pub fn generate_text(&mut self) {
		loop {
			let choice_stats = &self.stats.char_stats[self.current_order - 1].stats_for_state[&self.current[..]];

			self.update_order_used();
			self.calculate_distortions(&choice_stats);
			self.generate_next_character(&choice_stats);
		}
	}

	pub fn generate_sentence(&mut self) -> TextEvent {
		loop {
			let choice_stats = &self.stats.char_stats[self.current_order - 1].stats_for_state[&self.current[..]];

			self.update_order_used();
			self.calculate_distortions(&choice_stats);
			let event = self.generate_next_character(&choice_stats);

			match event {
				TextEvent::OutputComplete => return event,
				TextEvent::SentenceComplete(_) => return event,
				_ => continue,
			}
		}
	}

	pub fn pop_buffer_conents(&mut self) -> String {
		let contents = self.output_buffer.clone();
		self.output_buffer.clear();
		return contents;
	}

	fn update_order_used(&mut self) {
		if self.change_order_counter == 0 {
			if pick_random_in_range(0, 1) == 0 {
				if self.current_order > self.min_order {
					self.current_order -= 1;
				}
			} else {
				if self.current_order < self.max_order {
					self.current_order += 1;
				}
			}
			self.change_order_counter = 0;
		}
		else {
			self.change_order_counter += 1;
		}
	}

	fn calculate_distortions(&mut self, choice_stats: &CharChoiceStats) {
		self.distortions.total_usages = choice_stats.total_usages;
		self.distortions.options.clear();
		for (char_choice, count) in choice_stats.options.iter() {
			if self.sentence_watcher.enders.contains(char_choice) {
				let new_count = if self.current_sentence_length > self.sentence_watcher.word_count {
					(*count as f64 / self.distortion_factor as f64).ceil() as i32
				} else {
					count * self.distortion_factor
				};
				self.distortions.total_usages += new_count - count;
				self.distortions.options.insert(*char_choice, new_count);
			}
			// if *char_choice == '\n' {
			// 	let inside_paragraph = self.current_paragraph_length > self.paragraph_watcher.sentence_count;
			// 	let inside_sentence = self.sentence_watcher.word_count != 0;
			// 	let new_count = if inside_paragraph || inside_sentence {
			// 		(*count as f64 / self.distortion_factor as f64).ceil() as i32
			// 	} else {
			// 		count * self.distortion_factor
			// 	};
			// 	self.distortions.total_usages += new_count - count;
			// 	self.distortions.options.insert(*char_choice, new_count);
			// }
		}
	}

	fn generate_next_character(&mut self, choice_stats: &CharChoiceStats) -> TextEvent {
		let mut choice_num = pick_random_in_range(1, self.distortions.total_usages);

		for (next_char, next_count) in choice_stats.options.iter() {
			let mut count = *next_count;
			if let Some(count_override) = self.distortions.options.get(next_char) {
				count = *count_override;
			}
			choice_num -= count;

			if choice_num <= 0 {
				self.output(&next_char.to_string());

				self.current.push(*next_char);
				self.total += 1;

				let remove_count = cmp::max(self.current.chars().count() - self.current_order, 0);
				for _ in 0..remove_count {
					self.current.remove(0);
				}

				if let Some(word_count) = self.sentence_watcher.watch(*next_char) {
					if self.total >= self.output_amount {
						return TextEvent::OutputComplete;
					}
					self.generate_next_sentence_length();

					return TextEvent::SentenceComplete(word_count);
				}

				if let Some(sentence_count) = self.paragraph_watcher.watch(*next_char, &self.sentence_watcher) {

					let diff = (sentence_count - self.current_paragraph_length) as f64;
					let total = self.current_paragraph_length as f64;
					let percent = ((diff / total) * 100.0) as i32;
					println!("paragraph error percent: {}{}%", if percent >= 0 {" "} else {""}, percent);

					self.generate_next_paragraph_length();
					return TextEvent::ParagraphComplete(sentence_count);
				}

				break;
			}
		}

		return TextEvent::CharGenerated;
	}

	fn output(&mut self, string: &String) {
		if self.use_html {
			self.output_buffer.push_str("<span class=\"order-");
			self.output_buffer.push_str(&self.current.chars().count().to_string());
			self.output_buffer.push_str("\">");
		}

		self.output_buffer.push_str(&string);

		if self.use_html {
			self.output_buffer.push_str("</span>");
		}
	}

	fn generate_next_sentence_length(&mut self) {
		let stats_for_state = &self.stats.sentence_stats.stats_for_state;
		if let Some(choice_stats) = stats_for_state.get(&self.current_sentence_length) {
			let mut choice_num = pick_random_in_range(1, choice_stats.total_usages);

			for (next_length, count) in choice_stats.options.iter() {
				choice_num = choice_num - count;

				if choice_num <= 0 {
					self.current_sentence_length = *next_length;
					break;
				}
			}
		} else {
			let sentence_choice_index = pick_random_in_range(0, stats_for_state.len() - 1);
			self.current_sentence_length = *stats_for_state.keys().nth(sentence_choice_index).unwrap();
		}
	}

	fn generate_next_paragraph_length(&mut self) {
		let stats_for_state = &self.stats.paragraph_stats.stats_for_state;
		if let Some(choice_stats) = stats_for_state.get(&self.current_paragraph_length) {
			let mut choice_num = pick_random_in_range(1, choice_stats.total_usages);

			for (next_length, count) in choice_stats.options.iter() {
				choice_num = choice_num - count;

				if choice_num <= 0 {
					self.current_paragraph_length = *next_length;
					break;
				}
			}
		} else {
			let paragraph_choice_index = pick_random_in_range(0, stats_for_state.len() - 1);
			self.current_paragraph_length = *stats_for_state.keys().nth(paragraph_choice_index).unwrap();
		}
	}
}

#[allow(dead_code)]
fn debug_stats(stats: &OrderStats) {	
	// Print out stats:
	for (key, val) in stats.stats_for_state.iter() {
		println!("\"{}\":", key);

		for (key2, val2) in val.options.iter() {
			println!("   '{}' -> {}", key2, val2);
		}
	}
}

fn pick_random_in_range<T: NumCast>(start: T, end: T) -> T {
	let start_f = num::cast::<T, f64>(start).unwrap();
	let end_f = num::cast::<T, f64>(end).unwrap();

	let multiplier = end_f - start_f + 1.0;
	let r = rand::random::<f64>();
	let result = multiplier * r;
	
	return num::cast::<f64, T>(result).unwrap();
}
