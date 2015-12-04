
use std::cmp;
use gather_stats::OrderStats;

pub struct FormWatcher<'a> {
	// immutables:
	stats: &'a Vec<OrderStats<'a>>,
	max_order: usize,
	current: String,
	saw_possible_form_space: bool,
}

impl<'a> FormWatcher<'a> {
	pub fn new(stats: &'a Vec<OrderStats<'a>>, max_order: usize) -> FormWatcher<'a> {
		FormWatcher {
			stats: stats,
			max_order: max_order,
			current: String::new(),
			saw_possible_form_space: false,
		}
	}

	pub fn watch(&mut self, next_char: char) {

		// Add form entries to output if necessary:
		let mut changed = false;
		let last = self.current.chars().next_back();
		if next_char.is_alphabetic() || next_char == '-' {
			if last.is_none() || last.unwrap() != 'x' {
				self.current.push('x');
				changed = true;
			}
			self.saw_possible_form_space = false;
		} else {
			if next_char == ' ' && last.is_some() && last.unwrap() == 'x' {
				self.saw_possible_form_space = true;
			} else {
				if self.saw_possible_form_space {
					self.current.push(' ');
					self.saw_possible_form_space = false;
				}
				self.current.push(next_char);
				changed = true;
			}
		}

		let n = self.current.chars().count();
		let max = self.max_order;
		let remove_count = if n > max { n - max } else { 0 };
		for _ in 0..remove_count {
			self.current.remove(0);
		}

		if changed {
			let mut ord = self.current.chars().count() - 1;
			while !self.stats[ord].stats_for_state.contains_key(&self.current[..]) {
				self.current.remove(0);
				ord -= 1;
			}
			println!("{} {}", self.current, ord);
		}
	}
}
