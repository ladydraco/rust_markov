
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

const MAX_ORDER: usize = 3;

#[derive(Debug)]
struct OrderStats {
	total_usages: i32,
	options: HashMap<String, CharChoiceStats>
}

#[derive(Debug)]
struct CharChoiceStats {
	total_usages: i32,
	options: HashMap<char, i32>
}

fn main() {

	// load text as sting.

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
	for i in 0..MAX_ORDER {
		let mut order_stats = OrderStats {
			total_usages: 0,
			options: HashMap::new()
		};
		stats.push(order_stats);
	}

	// Iterate input text by character and extract statistics about each
	//  order as we go.

	let mut i = 0;
	let mut key = String::new();
	let mut window = String::new();
	for c in text.chars() {
		let mut next_char = c;

		// Look at the previous characters (prev_char) up to a max distance 
		//  of MAX_ORDER. For each order, add statistics for this
		//  following-character choice (next_char).

		if next_char == '\n' {
			next_char = ' ';
		}
		if next_char == '\r' {
			next_char = ' ';
		}

		key.clear();
		for (ord, prev_char) in window.chars().enumerate() {
			key.insert(0, prev_char);
			stats[ord].total_usages += 1;
			let mut no_match1 = false;
			if let Some(mut choice_stats) = stats[ord].options.get_mut(&key) {
				choice_stats.total_usages += 1;
				let mut no_match2 = false;
				if let Some(mut char_count) = choice_stats.options.get_mut(&next_char) {
					*char_count += 1;
				} else {
					no_match2 = true;
				}

				if no_match2 {
					choice_stats.options.insert(next_char, 1);
				}
			} else {
				no_match1 = true;
			}

			if no_match1 {
				let mut char_stats = HashMap::new();
				char_stats.insert(next_char, 1);
				let mut choice_stats = CharChoiceStats {
					total_usages: 1,
					options: char_stats
				};
				stats[ord].options.insert(key.clone(), choice_stats);
			}
		}

		move_window(&mut window, &next_char);

		// if i == 200 { break; }
		// i += 1;
	}

	// Print out stats for the third order:
	for (key, val) in stats[2].options.iter() {
		println!("\"{}\":", key);

		for (key2, val2) in val.options.iter() {
			println!("   '{}' -> {}", key2, val2);
		}
	}



	// Choose random starting string (encountered in the input text) 
	//  of length MAX_ORDER.



	// Generate characters that follow the starting string chosen by
	//  following random paths through the generated statistics.


    println!("Done.");
}

fn move_window(window: &mut String, next_char: & char) {
	window.insert(0, *next_char);
	if window.len() > MAX_ORDER {
		window.pop();
	}
}
