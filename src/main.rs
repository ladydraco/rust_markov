
extern crate rand;
extern crate num;

use std::cmp;
use std::env;
use std::process;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::collections::HashMap;
use std::collections::VecDeque;
use rand::random;
use num::traits::NumCast;

const INPUT_FILE: &'static str = "alice.txt";
const OUTPUT_FILE: &'static str = "output.txt";
const MIN_ORDER: usize = 3;
const MAX_ORDER: usize = 6;
const OUTPUT_CHARS: usize = 1200;

fn main() {
	let args = parse_arguments();

	let text = load_book(&args.input_filename);
	let stats = gather_statistics(&text, args.higher_order_bound);

	generate_text(&stats, &args);

	// debug_stats(&stats[args.higher_order_bound - 1]);

    println!("\nDone.");
}

#[derive(Debug)]
struct Args {
	input_filename: String,
	output_filename: String,
	lower_order_bound: usize,
	higher_order_bound: usize,
	output_amount: usize,
}

#[derive(Debug)]
struct OrderStats<'a> {
	total_usages: i32,
	previous_chars: HashMap<&'a str, CharChoiceStats>
}

impl<'a> OrderStats<'a> {
	fn add_stats(& mut self, key: &'a str, next_char: char) {
		self.total_usages += 1;

		if !self.previous_chars.contains_key(key) {
			let choice_stats = CharChoiceStats {
				total_usages: 0,
				options: HashMap::new()
			};
			self.previous_chars.insert(key, choice_stats);
		}

		let mut choice_stats = self.previous_chars.get_mut(key).unwrap();
		choice_stats.add_option(next_char);
	}
}

#[derive(Debug)]
struct CharChoiceStats {
	total_usages: i32,
	options: HashMap<char, i32>
}

trait AddOption<T> {
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

#[allow(dead_code)]
fn debug_stats(stats: &OrderStats) {	
	// Print out stats:
	for (key, val) in stats.previous_chars.iter() {
		println!("\"{}\":", key);

		for (key2, val2) in val.options.iter() {
			println!("   '{}' -> {}", key2, val2);
		}
	}
}

fn parse_arguments() -> Args {

	// Initialize args with default values:
	let mut parsed_args = Args {
		input_filename: String::from(INPUT_FILE),
		output_filename: String::from(OUTPUT_FILE),
		lower_order_bound: MIN_ORDER,
		higher_order_bound: MAX_ORDER,
		output_amount: OUTPUT_CHARS
	};

	for arg in env::args() {
		match &arg[0..2] {
			"-i" => parsed_args.input_filename = String::from(&arg[3..]),
			"-o" => parsed_args.output_filename = String::from(&arg[3..]),
			"-l" => parsed_args.lower_order_bound = parse_usize_or_default(&arg[3..], MIN_ORDER),
			"-h" => parsed_args.higher_order_bound = parse_usize_or_default(&arg[3..], MAX_ORDER),
			"-a" => parsed_args.output_amount = parse_usize_or_default(&arg[3..], OUTPUT_CHARS),
			"-?" => print_help(),
			_ => (),
		}
	}

	return parsed_args;
}

fn print_help() {
	println!("Arguments: ");
	println!(" -i: input filename.");
	println!(" -o: output filename.");
	println!(" -l: low order bound (minimum order to use).");
	println!(" -h: high order bound (maximum order to use).");
	println!(" -a: amount of generated output in characters.");
	println!(" -?: print help.");
	process::exit(1);
}

fn parse_usize_or_default(input: &str, default: usize) -> usize {
	if let Ok(arg_usize) = input.parse::<usize>() {
		arg_usize
	} else {
		default
	}
}

// Load the text of the book as a string.

fn load_book(file_name: &str) -> String {
	if let Ok(mut file) = File::open(file_name) {
		let mut file_contents = String::new();
		let _ = file.read_to_string(&mut file_contents);
		file_contents
	} else {
		panic!("Hey dumbass, there was a problem opening file.");
	}
}

// Build statistics which describe the probability of choosing 
//  a given character after a configurable (MAX_ORDER) number of
//  characters has been encountered.

fn gather_statistics(text: &str, max_order: usize) -> Vec<OrderStats> {
	let mut stats: Vec<OrderStats> = Vec::new();
	for _ in 0..max_order {
		let order_stats = OrderStats {
			total_usages: 0,
			previous_chars: HashMap::new()
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

fn generate_text(stats: &Vec<OrderStats>, args: &Args) {
	let mut output_file = if let Ok(file) = File::create(&args.output_filename) {
		file
	} else {
		panic!("Hey dumbass, there was a problem opening file.");
	};

	// Choose random starting string (encountered in the input text) 
	//  of length MAX_ORDER.

	let keys_count = stats[args.higher_order_bound - 1].previous_chars.len();
	let choice_index = pick_random_in_range(0, keys_count - 1);
	let mut current_ord = args.higher_order_bound;
	let mut current = String::from(*stats[current_ord - 1].previous_chars.keys().nth(choice_index).unwrap());
	let mut change_order_counter = 0;

	let _ = output_file.write(current.as_bytes());



	// Generate characters that follow the starting string chosen by
	//  following random paths through the generated statistics.

	let mut total = current.chars().count();

	loop {
		if total >= args.output_amount {
			break;
		}

		let choice_stats = &stats[current_ord - 1].previous_chars[&current[..]];

		update_order_used(&args, &mut change_order_counter, &mut current_ord);
		generate_next_character(&choice_stats, &current_ord, &mut current, &mut output_file, &mut total);
		
	}

	let _ = output_file.flush();
}

fn update_order_used(args: &Args, change_order_counter: &mut i32, current_ord: &mut usize) {
	if *change_order_counter == 0 {
		if pick_random_in_range(0, 1) == 0 {
			if *current_ord > args.lower_order_bound {
				*current_ord -= 1;
			}
		} else {
			if *current_ord < args.higher_order_bound {
				*current_ord += 1;
			}
		}
		*change_order_counter = 0;
	}
	else {
		*change_order_counter += 1;
	}
}

fn generate_next_character(choice_stats: &CharChoiceStats, current_ord: &usize, current: &mut String, output_file: &mut File, total: &mut usize) {
	let mut choice_num = pick_random_in_range(1, choice_stats.total_usages);

	for (next_char, count) in choice_stats.options.iter() {
		choice_num = choice_num - count;

		if choice_num <= 0 {
			let _ = output_file.write(next_char.to_string().as_bytes());
			current.push(*next_char);
			*total += 1;
			let remove_count = cmp::max(current.chars().count() - *current_ord, 0);
			for _ in 0..remove_count {
				current.remove(0);
			}
			break;
		}
	}
}

fn pick_random_in_range<T: NumCast>(start: T, end: T) -> T {
	let start_f = num::cast::<T, f64>(start).unwrap();
	let end_f = num::cast::<T, f64>(end).unwrap();

	let multiplier = end_f - start_f + 1.0;
	let r = rand::random::<f64>();
	let result = multiplier * r;
	
	return num::cast::<f64, T>(result).unwrap();
}
