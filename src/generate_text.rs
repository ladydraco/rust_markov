
use std::cmp;
use std::process;
use std::fs::File;
use std::io::Write;
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

#[derive(Debug)]
pub struct Args {
	pub input_filename: String,
	pub output_filename: String,
	pub lower_order_bound: usize,
	pub higher_order_bound: usize,
	pub output_amount: usize,
	pub use_html: bool
}

pub struct Generator<'a> {
	output_file: File,
	output_amount: usize,
	stats: &'a Stats<'a>,
	current: String,
	current_order: usize,
	total: usize,
	change_order_counter: i32,
	current_sentence_length: i32,
	distortions: CharChoiceStats,
	sentence_watcher: SentenceWatcher,
	use_html: bool,
}

impl<'a> Generator<'a> {
	pub fn new(args: &Args, stats: &'a Stats<'a>) -> Generator<'a> {
		let mut output_file = if let Ok(file) = File::create(&args.output_filename) {
			file
		} else {
			panic!("There was a problem opening the output file.");
		};

		if args.use_html {
			let _ = output_file.write("<meta charset=\"UTF-8\">".as_bytes());
			let _ = output_file.write("<style type=\"text/css\"> body { white-space: pre-wrap; } ".as_bytes());
			for i in args.lower_order_bound..args.higher_order_bound + 1 {
				
				let a = args.higher_order_bound + 1 - args.lower_order_bound;
				let b = i - args.lower_order_bound;
				let c = a - b - 1;
				let multiplier = c as f64 / a as f64;

				let value = (multiplier * 248.0) as i32;
				let _ = output_file.write(".order-".as_bytes());
				let _ = output_file.write(i.to_string().as_bytes());
				let _ = output_file.write("{ ".as_bytes());

				let _ = output_file.write(" color: rgb(".as_bytes());
				let _ = output_file.write(value.to_string().as_bytes());
				let _ = output_file.write(",".as_bytes());
				let _ = output_file.write(value.to_string().as_bytes());
				let _ = output_file.write(",".as_bytes());
				let _ = output_file.write(value.to_string().as_bytes());
				let _ = output_file.write(");\n".as_bytes());

				let _ = output_file.write("}\n".as_bytes());
			}
			let _ = output_file.write("</style>".as_bytes());
		}

		let mut generator = Generator {
			output_file: output_file,
			output_amount: args.output_amount,
			stats: stats,
			current: String::new(),
			current_order: args.higher_order_bound,
			total: 0,
			change_order_counter: 0,
			current_sentence_length: 0,
			distortions: CharChoiceStats {
				total_usages: 0,
				options: HashMap::new()
			},
			sentence_watcher: SentenceWatcher::new(),
			use_html: args.use_html
		};

		generator.start();

		return generator;
	}

	// Choose random starting string (encountered in the input text) 
	//  of length MAX_ORDER.

	fn start(&mut self) {
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
	}

	// Generate characters that follow the starting string chosen by
	//  following random paths through the generated statistics.

	pub fn generate_text(&mut self, args: &Args) {
		loop {
			let choice_stats = &self.stats.char_stats[self.current_order - 1].stats_for_state[&self.current[..]];

			self.update_order_used(&args);
			self.calculate_distortions(&choice_stats);
			self.generate_next_character(&choice_stats);
		}
	}

	fn update_order_used(&mut self, args: &Args) {
		if self.change_order_counter == 0 {
			if pick_random_in_range(0, 1) == 0 {
				if self.current_order > args.lower_order_bound {
					self.current_order -= 1;
				}
			} else {
				if self.current_order < args.higher_order_bound {
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
					(*count as f64 * 0.01).ceil() as i32
				} else {
					count * 100
				};
				self.distortions.total_usages += new_count - count;
				self.distortions.options.insert(*char_choice, new_count);
			}
			// if *char_choice == '\n' {
			// 	let new_count = (*count as f64 * 0.01).ceil() as i32;
			// 	self.distortions.total_usages += new_count - count;
			// 	self.distortions.options.insert(*char_choice, new_count);
			// }
		}
	}

	fn generate_next_character(&mut self, choice_stats: &CharChoiceStats) {
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

					let diff = num::abs(word_count - self.current_sentence_length) as f64;
					let total = self.current_sentence_length as f64;
					let percent = ((diff / total) * 100.0) as i32;
					println!("sentence error percent: {}%", percent);

					if self.total >= self.output_amount {
						let _ = self.output_file.flush();
    					println!("\nDone.");
						process::exit(1);
					}
					self.generate_next_sentence_length();
				}

				break;
			}
		}
	}

	fn output(&mut self, string: &String) {
		if self.use_html {
			let _ = self.output_file.write("<span class=\"order-".as_bytes());
			let _ = self.output_file.write(self.current.chars().count().to_string().as_bytes());
			let _ = self.output_file.write("\">".as_bytes());
		}

		let _ = self.output_file.write(string.as_bytes());

		if self.use_html {
			let _ = self.output_file.write("</span>".as_bytes());
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
