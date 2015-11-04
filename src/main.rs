
extern crate rand;

use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::collections::VecDeque;
use rand::random;

const MAX_ORDER: usize = 6;
const OUTPUT_CHARS: usize = 1200;

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

fn main() {
	let max_order = if let Some(arg) = env::args().nth(1) {
		if let Ok(argInt) = arg.parse::<usize>() {
			argInt
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
	}

	// Print out stats for the max order:
	// for (key, val) in stats[MAX_ORDER - 1].options.iter() {
	// 	println!("\"{}\":", key);

	// 	for (key2, val2) in val.options.iter() {
	// 		println!("   '{}' -> {}", key2, val2);
	// 	}
	// }



	// Choose random starting string (encountered in the input text) 
	//  of length MAX_ORDER.

	let choice_index = (rand::random::<f64>() * ((stats[max_order - 1].options.len() - 1) as f64)) as usize;
	let mut current = String::from(*stats[max_order - 1].options.keys().nth(choice_index).unwrap());
	print!("{}", current);



	// Generate characters that follow the starting string chosen by
	//  following random paths through the generated statistics.

	for _ in 0..(OUTPUT_CHARS - max_order - 1) {
		let choice_stats = &stats[max_order - 1].options[&current[..]];
		let total_usages = choice_stats.total_usages - 1;
		let mut choice_num = ((rand::random::<f64>() * (total_usages as f64)) as i32) + 1;

		for (next_char, count) in choice_stats.options.iter() {
			choice_num = choice_num - count;

			if choice_num <= 0 {
				print!("{}", next_char);
				current.push(*next_char);
				current.remove(0);
				break;
			}
		}
	}

    println!("\nDone.");
}
