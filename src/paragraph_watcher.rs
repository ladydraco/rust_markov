
use sentence_watcher::SentenceWatcher;

pub struct ParagraphWatcher {
	pub sentence_count: i32,
	in_sentence: bool,
	has_ended: bool,
	previous_length: Option<i32>,
}

impl ParagraphWatcher {
	pub fn new() -> ParagraphWatcher {
		ParagraphWatcher {
			sentence_count: 0,
			in_sentence: false,
			has_ended: false,
			previous_length: None,
		}
	}
	#[allow(dead_code)]
	pub fn sync(&mut self, target: &ParagraphWatcher) {
		self.sentence_count = target.sentence_count;
		self.previous_length = target.previous_length;
	}
	pub fn watch_for_stats(&mut self, next_char: char, sentence_watcher: &SentenceWatcher) -> Option<(Option<i32>, i32)> {
		let mut paragraph_ended = None;
		if sentence_watcher.start.is_none() {
			if self.in_sentence {
				self.in_sentence = false;
				self.sentence_count += 1;
			}
			if !self.has_ended && next_char == '\n' {
				self.has_ended = true;
				paragraph_ended = Some((self.previous_length, self.sentence_count));
				self.previous_length = Some(self.sentence_count);
				self.sentence_count = 0;
			}
		} else {
			if !self.in_sentence {
				self.in_sentence = true;
				self.has_ended = false;
			}
		}
		return paragraph_ended;
	}
}