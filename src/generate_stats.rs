
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct OrderStats<'a> {
	pub total_usages: i32,
	pub stats_for_state: HashMap<&'a str, CharChoiceStats>
}

impl<'a> OrderStats<'a> {
	fn add_stats(& mut self, key: &'a str, next_char: char) {
		self.total_usages += 1;

		if !self.stats_for_state.contains_key(key) {
			let choice_stats = CharChoiceStats {
				total_usages: 0,
				options: HashMap::new()
			};
			self.stats_for_state.insert(key, choice_stats);
		}

		let mut choice_stats = self.stats_for_state.get_mut(key).unwrap();
		choice_stats.add_option(next_char);
	}
}

#[derive(Debug)]
pub struct CharChoiceStats {
	pub total_usages: i32,
	pub options: HashMap<char, i32>
}

pub trait AddOption<T> {
	fn add_option(& mut self, T);
}

impl AddOption<char> for CharChoiceStats {
	fn add_option(& mut self, option: char) {
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

pub fn gather_statistics(text: &str, max_order: usize) -> Vec<OrderStats> {
	let mut stats: Vec<OrderStats> = Vec::new();
	for _ in 0..max_order {
		let order_stats = OrderStats {
			total_usages: 0,
			stats_for_state: HashMap::new()
		};
		stats.push(order_stats);
	}

	// Iterate input text by character and extract statistics about each
	//  order as we go.

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

		// Look at the previous characters up to a max distance 
		//  of MAX_ORDER. For each order, add statistics for this
		//  following-character choice (next_char).

		if window.len() > 1 {
			for i in 1..window.len() {
				// The order is one less than the slice distance of the key
				// for that order:
				let ord = i - 1;

				// Extract a key of length ord:
				let start = window[i];
				let end = window[0];
				let key = &text[start..end];
				
				stats[ord].add_stats(key, next_char);
			}
		}
	}

	return stats;
}
