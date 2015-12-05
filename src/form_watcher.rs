
use std::cmp;
use gather_stats::OrderStats;

pub struct FormWatcher<'a> {
	// immutables:
	stats: &'a Vec<OrderStats<'a>>,
	max_order: usize,
	current: String,
	pub current_order: usize,
	saw_possible_form_space: bool,
}

impl<'a> FormWatcher<'a> {
	pub fn new(stats: &'a Vec<OrderStats<'a>>, max_order: usize) -> FormWatcher<'a> {
		FormWatcher {
			stats: stats,
			max_order: max_order,
			current: String::new(),
			current_order: 0,
			saw_possible_form_space: false,
		}
	}

	pub fn sync(&mut self, target: &FormWatcher) {
		self.current.clear();
		self.current.push_str(&target.current);

		self.current_order = target.current_order;
		self.saw_possible_form_space = target.saw_possible_form_space;
	}

	pub fn watch(&mut self, next_char: char) -> (usize, bool) {

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

		let mut ord = self.current.chars().count() - 1;
		if changed {
			while !self.stats[ord].stats_for_state.contains_key(&self.current[..]) {
				self.current.remove(0);
				ord -= 1;
			}
			println!("{} {}", self.current, ord);
		}

		self.current_order = ord;

		return (ord, changed);
	}
}
