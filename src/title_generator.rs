
use std::collections::HashSet;
use generate_text::pick_random_in_range;
use regex::Regex;

#[derive(Debug,Eq,PartialEq,Hash,Copy,Clone)]
enum WordType {
	Noun(NounType),
	Pronoun,
	Adjective,
	Verb,
	Preposition,
	Article,
	Conjunction,
	Punctuation
}

#[derive(Debug,Eq,PartialEq,Hash,Copy,Clone)]
enum NounType {
	Person,
	Place,
	Thing
}

struct TitlePieces {
	words: WordsByType,
	templates: HashSet<Vec<WordType>>
}

struct WordsByType {
	nouns: NounsByType,
	pronouns: HashSet<&'static str>,
	adjectives: HashSet<&'static str>,
	verbs: HashSet<&'static str>,
	prepositions: HashSet<&'static str>,
	articles: HashSet<&'static str>,
	conjunctions: HashSet<&'static str>,
	punctuation: HashSet<&'static str>,
}

struct NounsByType {
	people: HashSet<&'static str>,
	places: HashSet<&'static str>,
	things: HashSet<&'static str>,
}

pub fn generate_title() -> String {
	let tp = define_title_pieces();
	let mut output = String::new();

	let template_choice = pick_random_in_range(0, tp.templates.len() - 1);
	let template = tp.templates.iter().nth(template_choice).unwrap();

	for word_type in template {
		let word = 
			match *word_type {
				WordType::Noun(noun_type) => 
					match noun_type {
						NounType::Person => pick(&tp.words.nouns.people),
						NounType::Place =>  pick(&tp.words.nouns.places),
						NounType::Thing =>  pick(&tp.words.nouns.things),
					},
				WordType::Pronoun =>     pick(&tp.words.pronouns),
				WordType::Adjective =>   pick(&tp.words.adjectives),
				WordType::Verb =>        pick(&tp.words.verbs),
				WordType::Preposition => pick(&tp.words.prepositions),
				WordType::Article =>     pick(&tp.words.articles),
				WordType::Conjunction => pick(&tp.words.conjunctions),
				WordType::Punctuation => pick(&tp.words.punctuation),
			};

		if output.len() > 0 {
			match *word_type {
				WordType::Punctuation => (),
				_ => output.push_str(" ")
			}
		}

		output.push_str(word);
	}

	let capitalized_first_letter = output.chars().nth(0).unwrap().to_uppercase().next().unwrap();
	let mut output2 = String::new();
	output2.push(capitalized_first_letter);
	output2.push_str(&output[1..]);

	let a_pattern = Regex::new(r"([Aa]) ([AEIOU])").unwrap();
	let output3 = a_pattern.replace_all(&output2, "$1n $2");

	return output3;
}

pub fn generate_author() -> String {
	let names = define_names();
	let mut author = String::new();
	author.push_str(pick(&names));
	author.push(' ');
	author.push_str(pick(&names));
	return author;
}

fn pick(words: & HashSet<&'static str>) -> &'static str {
	let choice = pick_random_in_range(0, words.len() - 1);
	return words.iter().nth(choice).unwrap();
}

fn define_title_pieces() -> TitlePieces {

	let nouns_people = {
		let mut set = HashSet::new();
		set.insert("Pig");
		set.insert("Rabbit");
		set.insert("Caterpillar");
		set.insert("Lobster");
		set.insert("Queen's");
		set.insert("Turtle's");
		set.insert("Alice's");
		set.insert("Bill");
		set
	};

	let nouns_places = {
		let mut set = HashSet::new();
		set.insert("Pool");
		set.insert("Croquet-Ground");
		set.insert("Rabbit-Hole");
		set.insert("Wonderland");
		set
	};

	let nouns_things = {
		let mut set = HashSet::new();
		set.insert("Tears");
		set.insert("Tale");
		set.insert("Advice");
		set.insert("Pepper");
		set.insert("Story");
		set.insert("Quadrille");
		set.insert("Tarts");
		set.insert("Evidence");
		set.insert("Caucus-Race");
		set.insert("Tea-Party");
		set.insert("Adventures");
		set
	};

	let pronouns = {
		let mut set = HashSet::new();
		set.insert("Who");
		set
	};

	let verbs = {
		let mut set = HashSet::new();
		set.insert("Sends");
		set.insert("Stole");
		set
	};

	let adjectives = {
		let mut set = HashSet::new();
		set.insert("Long");
		set.insert("Little");
		set.insert("Mad");
		set.insert("Mock");
		set
	};

	let prepositions = {
		let mut set = HashSet::new();
		set.insert("Down");
		set.insert("of");
		set.insert("in");
		set.insert("from");
		set
	};

	let articles = {
		let mut set = HashSet::new();
		set.insert("a");
		set.insert("the");
		set
	};

	let conjunctions = {
		let mut set = HashSet::new();
		set.insert("and");
		set
	};

	let punctuation = {
		let mut set = HashSet::new();
		set.insert("?");
		set
	};

	let templates = {
		let mut set = HashSet::new();
		set.insert(vec![
			WordType::Preposition, 
			WordType::Article,
			WordType::Noun(NounType::Place)
			]);

		set.insert(vec![
			WordType::Article,
			WordType::Noun(NounType::Place),
			WordType::Preposition, 
			WordType::Noun(NounType::Thing)
			]);

		set.insert(vec![
			WordType::Article,
			WordType::Noun(NounType::Thing),
			WordType::Conjunction, 
			WordType::Article, 
			WordType::Adjective, 
			WordType::Noun(NounType::Thing),
			]);

		set.insert(vec![
			WordType::Article,
			WordType::Noun(NounType::Person),
			WordType::Verb, 
			WordType::Preposition, 
			WordType::Article, 
			WordType::Adjective, 
			WordType::Noun(NounType::Person),
			]);

		set.insert(vec![
			WordType::Noun(NounType::Thing),
			WordType::Preposition, 
			WordType::Article,
			WordType::Noun(NounType::Person) 
			]);

		set.insert(vec![
			WordType::Noun(NounType::Person),
			WordType::Conjunction,
			WordType::Noun(NounType::Thing) 
			]);

		set.insert(vec![
			WordType::Article,
			WordType::Adjective,
			WordType::Noun(NounType::Thing) 
			]);

		set.insert(vec![
			WordType::Article,
			WordType::Noun(NounType::Person),
			WordType::Noun(NounType::Place)
			]);

		set.insert(vec![
			WordType::Article,
			WordType::Adjective,
			WordType::Noun(NounType::Person),
			WordType::Noun(NounType::Thing)
			]);

		set.insert(vec![
			WordType::Article,
			WordType::Noun(NounType::Person),
			WordType::Noun(NounType::Thing)
			]);

		set.insert(vec![
			WordType::Pronoun,
			WordType::Verb,
			WordType::Article,
			WordType::Noun(NounType::Thing),
			WordType::Punctuation,
			]);

		set.insert(vec![
			WordType::Noun(NounType::Person),
			WordType::Noun(NounType::Thing)
			]);

		set.insert(vec![
			WordType::Noun(NounType::Person),
			WordType::Noun(NounType::Thing),
			WordType::Preposition,
			WordType::Noun(NounType::Place),
			]);

		set
	};

	return TitlePieces {
		words: WordsByType {
			nouns: NounsByType {
				people: nouns_people,
				places: nouns_places,
				things: nouns_things
			},
			pronouns: pronouns,
			verbs: verbs,
			adjectives: adjectives,
			prepositions: prepositions,
			articles: articles,
			conjunctions: conjunctions,
			punctuation: punctuation
		},
		templates: templates
	};
}

fn define_names() -> HashSet<&'static str> {
	let mut set = HashSet::new();

	set.insert("Alice");
	set.insert("Antipathies");
	set.insert("Bill");
	set.insert("Canary");
	set.insert("Cat");
	set.insert("Caterpillar");
	set.insert("Cheshire");
	set.insert("Cat");
	set.insert("Conqueror");
	set.insert("Crab");
	set.insert("Dinah");
	set.insert("Dodo");
	set.insert("Dormouse");
	set.insert("Duchess");
	set.insert("Duck");
	set.insert("Eaglet");
	set.insert("Edgar");
	set.insert("Atheling");
	set.insert("Elsie");
	set.insert("Lacie");
	set.insert("Father");
	set.insert("William");
	set.insert("Fish-Footman");
	set.insert("Five");
	set.insert("Footman");
	set.insert("Frog-Footman");
	set.insert("Fury");
	set.insert("Gryphon");
	set.insert("Hatter");
	set.insert("Jack");
	set.insert("King");
	set.insert("Knave");
	set.insert("Lewis");
	set.insert("Carroll");
	set.insert("Little");
	set.insert("Bill");
	set.insert("Lizard");
	set.insert("Lobster");
	set.insert("Lory");
	set.insert("Magpie");
	set.insert("March");
	set.insert("Hare");
	set.insert("Mock");
	set.insert("Turtle");
	set.insert("Morcar");
	set.insert("Mouse");
	set.insert("Multiplication");
	set.insert("Northumbria");
	set.insert("Owl");
	set.insert("Panther");
	set.insert("Pepper");
	set.insert("Pigeon");
	set.insert("Queen");
	set.insert("Rabbit");
	set.insert("Seven");
	set.insert("Shakespeare");
	set.insert("Tillie");
	set.insert("Turtle");
	set.insert("Two");
	set.insert("White");
	set.insert("Rabbit");
	set.insert("William");

	return set;
}

