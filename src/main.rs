
extern crate rand;
extern crate num;

mod gather_stats;
mod generate_text;

use std::env;
use std::process;
use std::fs::File;
use std::io::Read;
use gather_stats::gather_statistics;
use generate_text::{
	Args,
	Generator,
	};

const INPUT_FILE: &'static str = "alice.txt";
const OUTPUT_FILE: &'static str = "output.txt";
const MIN_ORDER: usize = 3;
const MAX_ORDER: usize = 6;
const OUTPUT_CHARS: usize = 1200;

fn main() {
	let args = parse_arguments();

	let text = load_book(&args.input_filename);
	let stats = gather_statistics(&text, args.higher_order_bound);

	let mut generator = Generator::new(&args, &stats);
	generator.generate_text(&args);

    println!("\nDone.");
}

fn parse_arguments() -> Args {

	// Initialize args with default values:
	let mut parsed_args = Args {
		input_filename: String::from(INPUT_FILE),
		output_filename: String::from(OUTPUT_FILE),
		lower_order_bound: MIN_ORDER,
		higher_order_bound: MAX_ORDER,
		output_amount: OUTPUT_CHARS,
		use_html: false
	};

	for arg in env::args() {
		match &arg[0..2] {
			"-i" => parsed_args.input_filename = String::from(&arg[3..]),
			"-o" => parsed_args.output_filename = String::from(&arg[3..]),
			"-l" => parsed_args.lower_order_bound = parse_usize_or_default(&arg[3..], MIN_ORDER),
			"-h" => parsed_args.higher_order_bound = parse_usize_or_default(&arg[3..], MAX_ORDER),
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
	println!(" -a: amount of generated output in characters.");
	println!(" -f: print out html with color coding indicating the order.");
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
		panic!("There was a problem opening the input file.");
	}
}
