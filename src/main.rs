
extern crate rand;
extern crate num;
extern crate regex;

mod gather_stats;
mod generate_text;
mod preprocess;
mod form_watcher;

use std::env;
use std::process;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use gather_stats::gather_stats;
use generate_text::{
	Args,
	Generator,
	pick_random_in_range,
	};
use preprocess::{
	preprocess,
	extract_form
	};
use form_watcher::FormWatcher;
use regex::Regex;
use std::collections::HashSet;
use std::collections::VecDeque;

const INPUT_FILE: &'static str = "input/alice.txt";
const OUTPUT_FILE: &'static str = "output.txt";
const MIN_ORDER: usize = 3;
const MAX_ORDER: usize = 6;
const OUTPUT_CHARS: usize = 300;
const MAX_TRIES: usize = 5;
const DISTORTION_FACTOR: i32 = 10;
const FORM_MAX_ORDER: usize = 25;

fn main () {
	let args = parse_arguments();
	let raw_text = load_book(&args.input_filename);

	// Preprocess text, to disambuate what characters are content vs. form.

	let processed_text = preprocess(&raw_text);

	// Pick a starting point for the text generator: 

	// Find "max order" characters that begin a sentence.
	let mut text_starting_key = String::new();
	let sentence_ish_starts = Regex::new(r"[A-Z][\wʼ -]{14,}").unwrap();
	let matches = sentence_ish_starts.find_iter(&processed_text).collect::<Vec<_>>();
	let start_index = pick_random_in_range(0, matches.len() - 1);
	let start_bounds = matches[start_index];
	let start_match = &processed_text[start_bounds.0..start_bounds.1];
	let mut i = 0;
	for c in start_match.chars() {
		text_starting_key.push(c);
		i += 1;
		if i == args.higher_order_bound { break; }
	}

	let text = {
		let mut text = String::new();
		text.push_str(&processed_text);
		text.push_str(" ");
		text.push_str(&text_starting_key); // Append starting key, so book cannot end.
		text
	};

	let text_form = extract_form(&text);

	// Gather markov stats about both text and form:

	let text_stats = gather_stats(&text, args.higher_order_bound);
	let form_stats = gather_stats(&text_form, FORM_MAX_ORDER);

	// Create a generator for text:

	let mut output = String::new();
	let mut output_amount = 0;
	if args.use_html {
		write_html_header(&mut output, args.lower_order_bound, args.higher_order_bound);
	}

	// Create a form watcher for text:
	let mut watcher = FormWatcher::new(&form_stats, FORM_MAX_ORDER);

	let mut text_generator = Generator::new(&text_stats, &args, args.lower_order_bound, args.higher_order_bound);
	text_generator.start(Some(&text_starting_key));
	for c in text_starting_key.chars() { 
		output_char(&mut output, args.use_html, (c, args.higher_order_bound));
		watcher.watch(c);
		output_amount += 1;
	}

	while output_amount < args.output_amount {
		let next_item = text_generator.next();
		output_char(&mut output, args.use_html, next_item);
		watcher.watch(next_item.0);
		output_amount += 1;
	}

	output_file(&args.output_filename, &output);
}

fn write_html_header(output_buffer: &mut String, min_order: usize, max_order: usize) {
	output_buffer.push_str("<meta charset=\"UTF-8\">");
	output_buffer.push_str("<style type=\"text/css\"> body { white-space: pre-wrap; } ");
	output_buffer.push_str(".structure-success { background: #ddffdd; } ");
	for i in min_order..max_order + 1 {
		
		let a = max_order + 1 - min_order;
		let b = i - min_order;
		let c = a - b - 1;
		let multiplier = c as f64 / a as f64;

		let value = (multiplier * 248.0) as i32;
		output_buffer.push_str(".order-");
		output_buffer.push_str(&i.to_string());
		output_buffer.push_str("{ ");

		output_buffer.push_str(" color: rgb(");
		output_buffer.push_str(&value.to_string());
		output_buffer.push_str(",");
		output_buffer.push_str(&value.to_string());
		output_buffer.push_str(",");
		output_buffer.push_str(&value.to_string());
		output_buffer.push_str(");\n");

		output_buffer.push_str("}\n");
	}
	output_buffer.push_str("</style>");
}

fn output_char(output_buffer: &mut String, use_html: bool, next_output: (char, usize)) {
	if use_html {
		output_buffer.push_str("<span class=\"");
		output_buffer.push_str("order-");
		output_buffer.push_str(&next_output.1.to_string());
		output_buffer.push_str("\">");
	}

	output_buffer.push(next_output.0);

	if use_html {
		output_buffer.push_str("</span>");
	}
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
