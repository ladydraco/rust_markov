
use regex::Regex;
use std::collections::VecDeque;

pub fn preprocess(input: &String) -> String {
	let contraction_pattern = Regex::new(r"(\w)'(\w)").unwrap();
	let leftover_pattern    = Regex::new(r"'").unwrap();
	let dash_pattern        = Regex::new(r"--").unwrap();

	let text1 = input;
	let text2 = contraction_pattern.replace_all(&text1, "$1\u{02BC}$2");
	let text3 = process_quotes(&text2);
	let text4 = leftover_pattern.replace_all(&text3, "\u{02BC}");
	let text5 = dash_pattern.replace_all(&text4, "\u{2014}");

	return text5;
}

fn process_quotes(input: &String) -> String {
	let quotes_pattern = Regex::new(r"([^\w])'((?s).*?)'([^\w])").unwrap();
	let mut open_quotes = VecDeque::new();
	let mut close_quotes = VecDeque::new();
	let mut search_text = &input[..];
	let mut search_text_offset = 0;

	// Locate opening and closing quotes:

	while let Some((a, b)) = quotes_pattern.find(search_text) {

		// Create an iterator over char indices of match.
		let slice = &input[a..b];
		let mut slice_chars = slice.char_indices();

		// Mark the opening quote apostrophe:
		let _ = slice_chars.next().unwrap();
		let (open_quote, _) = slice_chars.next().unwrap();
		open_quotes.push_front(search_text_offset + a + open_quote);

		// Find the closing quote apostrophe:
		let close_quote = {
			let mut offset2 = open_quote;
			let (mut offset1, _) = slice_chars.next().unwrap();

			while let Some((offset, _)) = slice_chars.next() {
				offset2 = offset1;
				offset1 = offset;
			}
			offset2
		};
		close_quotes.push_front(search_text_offset + a + close_quote);

		// Move search text window:
		search_text_offset = search_text_offset + a + close_quote + 1;
		search_text = &input[search_text_offset..];
	}

	// Substitute in the proper unicode character for them:

	let mut output = String::new();
	let mut open_quote = open_quotes.pop_back();
	let mut close_quote = close_quotes.pop_back();
	for (i, c) in input.char_indices() {
		if open_quote.is_some() && open_quote.unwrap() == i {
			output.push('\u{2018}');
			open_quote = open_quotes.pop_back();
		} else if close_quote.is_some() && close_quote.unwrap() == i {
			output.push('\u{2019}');
			close_quote = close_quotes.pop_back();
		} else {
			output.push(c);
		}
	}

	return output;
}

pub fn extract_form(processed_text: &String) -> String {
	let mut output = String::new();

	let mut saw_alphabetic = false;

	for c in processed_text.chars() {
		if c.is_alphabetic() || c == '-' {
			saw_alphabetic = true;
		} else {
			if saw_alphabetic {
				output.push('x');
			}
			output.push(c);
			saw_alphabetic = false;
		}
	}

	let grouping_pattern = Regex::new(r"x( x)*").unwrap();
	let output2 = grouping_pattern.replace_all(&output, "x");

	return output2;
}

