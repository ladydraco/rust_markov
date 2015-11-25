
use std::collections::HashMap;
use std::collections::VecDeque;
use sentence_watcher::SentenceWatcher;
use paragraph_watcher::ParagraphWatcher;

#[derive(Debug)]
pub struct Stats<'a> {
	pub char_stats: Vec<OrderStats<'a>>,
	pub sentence_stats: SentenceStats<'a>,
	pub paragraph_stats: ParagraphStats,
}

impl<'a> Stats<'a> {
	fn new(max_order: usize) -> Stats<'a> {
		let mut char_stats: Vec<OrderStats> = Vec::new();
		for _ in 0..max_order {
			let order_stats = OrderStats {
				total_usages: 0,
				stats_for_state: HashMap::new()
			};
			char_stats.push(order_stats);
		}

		let sentence_stats = SentenceStats {
			total_usages: 0,
			starts: Vec::new(),
			stats_for_state: HashMap::new()
		};

		let paragraph_stats = ParagraphStats {
			total_usages: 0,
			stats_for_state: HashMap::new()
		};

		Stats {
			char_stats: char_stats,
			sentence_stats: sentence_stats,
			paragraph_stats: paragraph_stats,
		}
	}
}



#[derive(Debug)]
pub struct OrderStats<'a> {
	pub total_usages: i32,
	pub stats_for_state: HashMap<&'a str, CharChoiceStats>
}

const USAGE_MULTIPLIER: i32 = 10;

impl<'a> OrderStats<'a> {
	fn add_stats(&mut self, state: &'a str, next: char) {
		self.total_usages += 1 * USAGE_MULTIPLIER;

		if !self.stats_for_state.contains_key(state) {
			let choice_stats = CharChoiceStats {
				total_usages: 0,
				options: HashMap::new()
			};
			self.stats_for_state.insert(state, choice_stats);
		}

		let mut choice_stats = self.stats_for_state.get_mut(state).unwrap();
		choice_stats.add_option(next);
	}
}

#[derive(Debug)]
pub struct CharChoiceStats {
	pub total_usages: i32,
	pub options: HashMap<char, i32>
}

impl CharChoiceStats {
	fn add_option(&mut self, option: char) {
		self.total_usages += 1 * USAGE_MULTIPLIER;

		if !self.options.contains_key(&option) {
			self.options.insert(option, 0);
		}

		let mut char_count = self.options.get_mut(&option).unwrap();
		*char_count += 1 * USAGE_MULTIPLIER;
	}
}



#[derive(Debug)]
pub struct SentenceStats<'a> {
	pub total_usages: i32,
	pub starts: Vec<&'a str>,
	pub stats_for_state: HashMap<i32, SentenceChoiceStats>
}

impl<'a> SentenceStats<'a> {
	fn add_stats(&mut self, state: i32, next: i32) {
		self.total_usages += 1;

		if !self.stats_for_state.contains_key(&state) {
			let choice_stats = SentenceChoiceStats {
				total_usages: 0,
				options: HashMap::new()
			};
			self.stats_for_state.insert(state, choice_stats);
		}

		let mut choice_stats = self.stats_for_state.get_mut(&state).unwrap();
		choice_stats.add_option(next);
	}
}

#[derive(Debug)]
pub struct SentenceChoiceStats {
	pub total_usages: i32,
	pub options: HashMap<i32, i32>
}

impl SentenceChoiceStats {
	fn add_option(&mut self, option: i32) {
		self.total_usages += 1;

		if !self.options.contains_key(&option) {
			self.options.insert(option, 0);
		}

		let mut char_count = self.options.get_mut(&option).unwrap();
		*char_count += 1;
	}
}


#[derive(Debug)]
pub struct ParagraphStats {
	pub total_usages: i32,
	pub stats_for_state: HashMap<i32, ParagraphChoiceStats>
}

impl ParagraphStats {
	fn add_stats(&mut self, state: i32, next: i32) {
		self.total_usages += 1;

		if !self.stats_for_state.contains_key(&state) {
			let choice_stats = ParagraphChoiceStats {
				total_usages: 0,
				options: HashMap::new()
			};
			self.stats_for_state.insert(state, choice_stats);
		}

		let mut choice_stats = self.stats_for_state.get_mut(&state).unwrap();
		choice_stats.add_option(next);
	}
}

#[derive(Debug)]
pub struct ParagraphChoiceStats {
	pub total_usages: i32,
	pub options: HashMap<i32, i32>
}

impl ParagraphChoiceStats {
	fn add_option(&mut self, option: i32) {
		self.total_usages += 1;

		if !self.options.contains_key(&option) {
			self.options.insert(option, 0);
		}

		let mut char_count = self.options.get_mut(&option).unwrap();
		*char_count += 1;
	}
}


// Build statistics which describe the probability of choosing 
//  a given character after a configurable (MAX_ORDER) number of
//  characters has been encountered.

pub fn gather_statistics(text: &str, max_order: usize) -> Stats {

	let mut stats = Stats::new(max_order);

	// Iterate input text by character and extract statistics about each
	//  order as we go.

	let mut sentence_watcher = SentenceWatcher::new();
	let mut paragraph_watcher = ParagraphWatcher::new();

	// A sliding window of length MAX_ORDER + 1 that captures the character offsets
	//  of current character as well as the past MAX_ORDER characters. This allows
	//  us to create <&str> representations of strings of the previous 1, 2, ..., MAX_ORDER
	//  characters, in order to gather statistics about how likely the current character
	//  is to follow them.
	let mut window = VecDeque::new();

	for (offset, next_char) in text.char_indices() {

		// Move the window (so that it includes the current character's offset).

		window.push_front(offset);
		if window.len() > max_order + 1 {
			window.pop_back();
		}

		// Collect character-level stats for each order:

		if window.len() > 1 {
			for i in 1..window.len() {
				// The order is one less than the slice distance of the key
				// for that order:
				let ord = i - 1;

				// Extract a key of length ord:
				let start = window[i];
				let end = window[0];
				let key = &text[start..end];
				
				stats.char_stats[ord].add_stats(key, next_char);
			}
		}

		// Collect sentence-length stats and mark valid sentence beginnings:

		if let Some(sentence_ended) = sentence_watcher.watch_for_stats(offset, next_char) {
			let (word_count, start, previous_length) = sentence_ended;

			stats.sentence_stats.starts.push(&text[start..]);

			if let Some(previous_length) = previous_length {
				stats.sentence_stats.add_stats(previous_length, word_count);
			}
		}

		// Collection paragraph-length stats:

		if let Some((previous_length, sentence_count)) = paragraph_watcher.watch_for_stats(next_char, &sentence_watcher) {
			if let Some(previous_length) = previous_length {
				stats.paragraph_stats.add_stats(previous_length, sentence_count);
			}
		}
	}

	// Print out paragraph stats:
	for (key, val) in stats.paragraph_stats.stats_for_state.iter() {
		println!("\"{}\":", key);

		for (key2, val2) in val.options.iter() {
			println!("   '{}' -> {}", key2, val2);
		}
	}

	return stats;
}
