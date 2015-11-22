
extern crate rand;
extern crate num;

mod gather_stats;
mod generate_text;
mod sentence_watcher;

use std::env;
use std::process;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use gather_stats::gather_statistics;
use generate_text::{
	Args,
	Generator,
	TextEvent,
	};

const INPUT_FILE: &'static str = "alice.txt";
const OUTPUT_FILE: &'static str = "output.txt";
const MIN_ORDER: usize = 3;
const MAX_ORDER: usize = 6;
const OUTPUT_CHARS: usize = 1200;
const MAX_TRIES: usize = 5;
const DISTORTION_FACTOR: i32 = 10;

fn main() {
	let args = parse_arguments();

	let text = load_book(&args.input_filename);
	let stats = gather_statistics(&text, args.higher_order_bound);

	let mut generator = Generator::new(&args, &stats);
	generator.start();

	let mut slaves = Vec::new();
	for _ in 0..args.max_tries {
		slaves.push(Generator::new(&args, &stats));
	}

	let mut output = String::new();
	loop {
		let desired_word_count = generator.current_sentence_length;
		let mut word_count_mismatch = std::i32::MAX;
		let mut winning_slave = None;
		let mut winning_event = TextEvent::CharGenerated;

		for i in 0..args.max_tries {
			slaves[i].sync(&generator);
			let event = slaves[i].generate_sentence();

			if let TextEvent::SentenceComplete(word_count) = event {
				let diff = num::abs(word_count - desired_word_count);
				if diff < word_count_mismatch {
					word_count_mismatch = diff;
					winning_slave = Some(i);
					winning_event = event;
				}
			}

			if let TextEvent::OutputComplete = event {
				winning_slave = Some(i);
				winning_event = event;
				break;
			}
		}

		if let Some(winner) = winning_slave {
			generator.sync(&slaves[winner]);

			let sentence = generator.pop_buffer_conents();
			output.push_str(&sentence);

			if let TextEvent::SentenceComplete(word_count) = winning_event {
				let diff = (word_count - desired_word_count) as f64;
				let total = desired_word_count as f64;
				let percent = ((diff / total) * 100.0) as i32;
				println!("sentence error percent: {}{}%", if percent >= 0 {" "} else {""}, percent);
			}

			if let TextEvent::OutputComplete = winning_event {
				break;
			}
		}
		
	}

	output_file(&args.output_filename, &output);

	println!("\nDone.");
}

fn parse_arguments() -> Args {

	// Initialize args with default values:
	let mut parsed_args = Args {
		input_filename: String::from(INPUT_FILE),
		output_filename: String::from(OUTPUT_FILE),
		lower_order_bound: MIN_ORDER,
		higher_order_bound: MAX_ORDER,
		max_tries: MAX_TRIES,
		distortion_factor: DISTORTION_FACTOR,
		output_amount: OUTPUT_CHARS,
		use_html: false
	};

	for arg in env::args() {
		match &arg[0..2] {
			"-i" => parsed_args.input_filename = String::from(&arg[3..]),
			"-o" => parsed_args.output_filename = String::from(&arg[3..]),
			"-l" => parsed_args.lower_order_bound = parse_usize_or_default(&arg[3..], MIN_ORDER),
			"-h" => parsed_args.higher_order_bound = parse_usize_or_default(&arg[3..], MAX_ORDER),
			"-t" => parsed_args.max_tries = parse_usize_or_default(&arg[3..], MAX_TRIES),
			"-d" => parsed_args.distortion_factor = parse_i32_or_default(&arg[3..], DISTORTION_FACTOR),
			"-a" => parsed_args.output_amount = parse_usize_or_default(&arg[3..], OUTPUT_CHARS),
			"-f" => parsed_args.use_html = true,
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
	println!(" -d: distortion factor, how much to distort statistics to achieve structure goals (1-10).");
	println!(" -t: tries, how many times to try generating the desired output to achieve structure goals (1-10).");
	println!(" -a: amount of generated output in characters.");
	println!(" -f: format as html with color coding indicating the order.");
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

fn parse_i32_or_default(input: &str, default: i32) -> i32 {
	if let Ok(arg_i32) = input.parse::<i32>() {
		arg_i32
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
		panic!("There was a problem opening the input file.");
	}
}

fn output_file(file_name: &str, output: &String) {
	if let Ok(mut file) = File::create(file_name) {
		let _ = file.write(output.as_bytes());
		let _ = file.flush();
	} else {
		panic!("There was a problem opening the output file.");
	}
}
