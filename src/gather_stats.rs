
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct OrderStats<'a> {
	pub total_usages: i32,
	pub stats_for_state: HashMap<&'a str, CharChoiceStats>
}

impl<'a> OrderStats<'a> {
	fn add_stats(&mut self, state: &'a str, next: char) {
		self.total_usages += 1;

		self.stats_for_state
			.entry(state)
			.or_insert_with(|| CharChoiceStats {
				total_usages: 0,
				options: HashMap::new()
			})
			.add_option(next);
	}
}

#[derive(Debug)]
pub struct CharChoiceStats {
	pub total_usages: i32,
	pub options: HashMap<char, i32>
}

impl CharChoiceStats {
	fn add_option(&mut self, option: char) {
		self.total_usages += 1;

		*self.options
			.entry(option)
			.or_insert(0) += 1;
	}
}


// Build statistics which describe the probability of choosing 
//  a given character after a configurable (MAX_ORDER) number of
//  characters has been encountered.

pub fn gather_stats(text: &str, max_order: usize) -> Vec<OrderStats> {
	let mut stats: Vec<OrderStats> = Vec::new();
	for _ in 0..max_order {
		let order_stats = OrderStats {
			total_usages: 0,
			stats_for_state: HashMap::new()
		};
		stats.push(order_stats);
	}

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

	return stats;
}
