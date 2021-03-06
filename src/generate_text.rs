
use std::cmp;
use std::collections::HashMap;
use rand;
use rand::random;
use num;
use num::traits::NumCast;
use gather_stats::{OrderStats, CharChoiceStats};

#[derive(Debug)]
pub struct Args {
    pub input_filename: String,
    pub output_filename: String,
    pub lower_order_bound: usize,
    pub higher_order_bound: usize,
    pub max_tries: usize,
    pub distortion_factor: i32,
    pub output_amount: usize,
    pub use_html: bool,
}

pub struct Generator<'a> {
    // immutables:
    stats: &'a Vec<OrderStats<'a>>,
    max_order: usize,
    min_order: usize,
    output_amount: usize,
    use_html: bool,
    distortion_factor: i32,

    // current state:
    //
    // char-level:
    current: String,
    current_order: usize,
    total: usize,
    change_order_counter: i32,
    distortions: CharChoiceStats,
}

impl<'a> Generator<'a> {
    pub fn new(stats: &'a Vec<OrderStats<'a>>,
               args: &Args,
               min_order: usize,
               max_order: usize)
               -> Generator<'a> {

        let generator = Generator {
            stats: stats,
            max_order: max_order,
            min_order: min_order,
            output_amount: args.output_amount,
            use_html: args.use_html,
            distortion_factor: args.distortion_factor,

            current: String::new(),
            current_order: max_order,
            total: 0,
            change_order_counter: 0,
            distortions: CharChoiceStats {
                total_usages: 0,
                options: HashMap::new(),
            },
        };

        return generator;
    }

    pub fn sync(&mut self, target: &Generator) {
        self.current.clear();

        self.current.push_str(&target.current);

        self.current_order = target.current_order;
        self.total = target.total;
        self.change_order_counter = target.change_order_counter;
    }

    // Choose random starting string (encountered in the input text)
    //  of length MAX_ORDER.

    pub fn start(&mut self, start: Option<&str>) {
        if start.is_some() {
            self.current = String::from(start.unwrap());
        } else {
            let start_index = pick_random_in_range(0,
                                                   self.stats[self.current_order - 1]
                                                       .stats_for_state
                                                       .len() -
                                                   1);
            self.current = String::from(*self.stats[self.current_order - 1]
                                             .stats_for_state
                                             .keys()
                                             .nth(start_index)
                                             .unwrap());
        }
    }

    fn update_order_used(&mut self) {
        if self.change_order_counter == 0 {
            if pick_random_in_range(0, 1) == 0 {
                if self.current_order > self.min_order {
                    self.current_order -= 1;
                }
            } else {
                if self.current_order < self.max_order {
                    self.current_order += 1;
                }
            }
            self.change_order_counter = 0;
        } else {
            self.change_order_counter += 1;
        }
    }

    fn calculate_distortions(&mut self, choice_stats: &CharChoiceStats) {
        self.distortions.total_usages = choice_stats.total_usages;
        self.distortions.options.clear();
        for (char_choice, count) in choice_stats.options.iter() {
            // if self.sentence_watcher.enders.contains(char_choice) {
            // 	let new_count = if self.current_sentence_length > self.sentence_watcher.word_count {
            // 		(*count as f64 / self.distortion_factor as f64).ceil() as i32
            // 	} else {
            // 		count * self.distortion_factor
            // 	};
            // 	self.distortions.total_usages += new_count - count;
            // 	self.distortions.options.insert(*char_choice, new_count);
            // }
        }
    }

    pub fn next(&mut self) -> (char, usize) {
        let order_used = self.current_order;
        let choice_stats = if let Some(choice_stats) = self.stats[self.current_order - 1]
                                                           .stats_for_state
                                                           .get(&self.current[..]) {
            choice_stats
        } else {
            panic!("Current state could not be found in stats. Key: '{}' (length: {}), Order: {}",
                   self.current,
                   self.current.chars().count(),
                   self.current_order);
        };

        self.update_order_used();
        self.calculate_distortions(&choice_stats);

        let mut choice_num = pick_random_in_range(1, self.distortions.total_usages);

        for (next_char, next_count) in choice_stats.options.iter() {
            let mut count = *next_count;
            if let Some(count_override) = self.distortions.options.get(next_char) {
                count = *count_override;
            }
            choice_num -= count;

            if choice_num <= 0 {
                self.current.push(*next_char);
                self.total += 1;

                let remove_count = cmp::max(self.current.chars().count() - self.current_order, 0);
                for _ in 0..remove_count {
                    self.current.remove(0);
                }

                return (*next_char, order_used);
            }
        }

        panic!("Failed to choose a next character.");
    }
}

#[allow(dead_code)]
fn debug_stats(stats: &OrderStats) {
    // Print out stats:
    for (key, val) in stats.stats_for_state.iter() {
        println!("\"{}\":", key);

        for (key2, val2) in val.options.iter() {
            println!("   '{}' -> {}", key2, val2);
        }
    }
}

pub fn pick_random_in_range<T: NumCast>(start: T, end: T) -> T {
    let start_f = num::cast::<T, f64>(start).unwrap();
    let end_f = num::cast::<T, f64>(end).unwrap();

    let multiplier = end_f - start_f + 1.0;
    let r = rand::random::<f64>();
    let result = start_f + (multiplier * r);

    return num::cast::<f64, T>(result).unwrap();
}
