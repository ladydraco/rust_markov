
extern crate rand;
extern crate num;

use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use rand::random;
use num::traits::NumCast;

const MAX_ORDER: usize = 6;
const OUTPUT_CHARS: usize = 800;

#[derive(Debug)]
struct OrderStats<'a> {
	total_usages: i32,
	options: HashMap<&'a str, CharChoiceStats>
}

#[derive(Debug)]
struct CharChoiceStats {
	total_usages: i32,
	options: HashMap<char, i32>
}

#[derive(Debug)]
struct SentenceChoiceStats {
	total_usages: i32,
	options: HashMap<i32, i32>
}

fn main() {
	let max_order = if let Some(arg) = env::args().nth(1) {
		if let Ok(arg_int) = arg.parse::<usize>() {
			arg_int
		} else {
			MAX_ORDER
		}
	} else {
		MAX_ORDER
	};


	// Load the text of the book as a sting.

	let text = if let Ok(mut file) = File::open("alice.txt") {
		let mut file_contents = String::new();
		let _ = file.read_to_string(&mut file_contents);
		file_contents
	} else {
		panic!("Hey dumbass, there was a problem opening file.");
	};



	// Build statistics which describe the probability of choosing 
	//  a given character after a configurable (MAX_ORDER) number of
	//  characters has been encountered.

	let mut stats: Vec<OrderStats> = Vec::new();
	for _ in 0..max_order {
		let order_stats = OrderStats {
			total_usages: 0,
			options: HashMap::new()
		};
		stats.push(order_stats);
	}

	// Iterate input text by character and extract statistics about each
	//  order as we go.

	let mut sentences: Vec<&str> = Vec::new();
	let mut current_word_count = 0;
	let mut in_word = false;
	let mut sentence_start = -1;

	let mid_word_punctuation = {
		let mut mid_word_punctuation = HashSet::new();
		mid_word_punctuation.insert('\'');
		mid_word_punctuation.insert('-');
		mid_word_punctuation
	};

	let sentence_enders = {
		let mut sentence_enders = HashSet::new();
		sentence_enders.insert('.');
		sentence_enders.insert('!');
		sentence_enders.insert('?');
		sentence_enders
	};

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
				
				if stats[ord].options.contains_key(key) {
					let mut choice_stats = stats[ord].options.get_mut(key).unwrap();
					choice_stats.total_usages += 1;

					if choice_stats.options.contains_key(&next_char) {
						// We have found another occurrence of next_char following the
						//  previous sequence of characters (key). Increment the count.

						let mut char_count = choice_stats.options.get_mut(&next_char).unwrap();
						*char_count += 1;
					} else {
						// This is the first occurrence of next_char following this
						//  sequence of characters (key). Insert a counter for it.

						choice_stats.options.insert(next_char, 1);
					}
				} else {
					// We have found the first occurrence of a string of whatever order
					//  we are currently handling (ord), in the text. We will create
					//  a new CharChoiceStats to track single character choices that
					//  follow this string sequence.

					let mut char_stats = HashMap::new();
					char_stats.insert(next_char, 1);
					let choice_stats = CharChoiceStats {
						total_usages: 1,
						options: char_stats
					};
					stats[ord].options.insert(key, choice_stats);
				}
			}
		}

		// Collect stats about sentence lengths and mark the beginning
		//  of sentences.

		if next_char.is_alphabetic() {
			if !in_word {
				in_word = true;
			}
			if sentence_start == -1 {
				sentence_start = offset;
				current_word_count = 0;
			}
		} else {
			if in_word {
				in_word = false;
				current_word_count += 1;
			}
			if sentence_enders.contains(&next_char) {
				if sentence_start != -1 {
					sentences.push(&text[sentence_start..offset+next_char.len_utf8()]);
					sentence_start = -1;
				}
			}
			if next_char == '\n' {
				sentence_start = -1;
			}
		}
	}

	// Print out stats for the max order:
	// for (key, val) in stats[MAX_ORDER - 1].options.iter() {
	// 	println!("\"{}\":", key);

	// 	for (key2, val2) in val.options.iter() {
	// 		println!("   '{}' -> {}", key2, val2);
	// 	}
	// }

	// println!("{}", sentences.len());
	// let mut i = 0;
	// for s in sentences.iter() {
	// 	println!("{}", s);
	// 	println!("----");
	// 	i += 1;
	// 	if i == 20 {
	// 		break;
	// 	}
	// }



	// Choose random starting string (encountered in the input text) 
	//  of length MAX_ORDER.

	let keys_count = stats[max_order - 1].options.len();
	let choice_index = pick_random_in_range(0, keys_count - 1);
	let mut current_ord = max_order;
	let mut current = String::from(*stats[current_ord - 1].options.keys().nth(choice_index).unwrap());
	let mut change_order_counter = 0;
	// print!("{}", current);



	// Generate characters that follow the starting string chosen by
	//  following random paths through the generated statistics.

	for _ in 0..(OUTPUT_CHARS - max_order - 2) {
		break;

		let choice_stats = &stats[current_ord - 1].options[&current[..]];
		let mut choice_num = pick_random_in_range(1, choice_stats.total_usages);

		if change_order_counter == 0 {
			if pick_random_in_range(0, 1) == 0 {
				if current_ord > 3 {
					current_ord -= 1;
				}
			} else {
				if current_ord < max_order {
					current_ord += 1;
				}
			}
			change_order_counter = 0;
		}
		else {
			change_order_counter += 1;
		}


		for (next_char, count) in choice_stats.options.iter() {
			choice_num = choice_num - count;

			if choice_num <= 0 {
				print!("{}", next_char);
				current.push(*next_char);
				while current.len() > current_ord {
					current.remove(0);
				}
				break;
			}
		}
	}

    println!("\nDone.");
}

fn pick_random_in_range<T: NumCast>(start: T, end: T) -> T {
	let start_f = num::cast::<T, f64>(start).unwrap();
	let end_f = num::cast::<T, f64>(end).unwrap();

	let multiplier = end_f - start_f + 1.0;
	let r = rand::random::<f64>();
	let result = multiplier * r;
	
	return num::cast::<f64, T>(result).unwrap();
}
