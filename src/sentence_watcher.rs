
use std::collections::HashSet;

pub struct SentenceWatcher {
	pub word_count: i32,
	pub enders: HashSet<char>,
	
	in_word: bool,
	start: Option<usize>,
	previous_length: Option<i32>,
}

impl SentenceWatcher {
	pub fn new() -> SentenceWatcher {
		let mut enders = HashSet::new();
		enders.insert('.');
		enders.insert('!');
		enders.insert('?');

		// 	let mut mid_word_punctuation = HashSet::new();
		// 	mid_word_punctuation.insert('\'');
		// 	mid_word_punctuation.insert('-');

		SentenceWatcher {
			word_count: 0,
			in_word: false,
			start: None,
			previous_length: None,
			enders: enders
		}
	}
	pub fn watch_for_stats(&mut self, offset: usize, next_char: char) -> Option<(i32, usize, Option<i32>)> {
		let mut sentence_ended = None;
		if next_char.is_alphabetic() {
			if !self.in_word {
				self.in_word = true;
			}
			if self.start.is_none() {
				self.start = Some(offset);
			}
		} else {
			if self.in_word {
				self.in_word = false;
				self.word_count += 1;
			}
			if self.enders.contains(&next_char) {
				if let Some(start) = self.start {
					sentence_ended = Some((self.word_count, start, self.previous_length));
					self.previous_length = Some(self.word_count);
					self.word_count = 0;
					self.start = None;
				}
			}
			if next_char == '\n' {
				self.start = None;
			}
		}
		return sentence_ended;
	}
	pub fn watch(&mut self, next_char: char) -> Option<i32> {
		if let Some((word_count, _, _)) = self.watch_for_stats(0, next_char) {
			Some(word_count)
		} else {
			None
		}
	}
}
