use std::collections::HashMap;

use ahash::RandomState;
use okkhor::parser::Parser;
use peak_alloc::PeakAlloc;
use regex::Regex;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let Some(word) = std::env::args().nth(1) else {
        eprintln!("Please provide a word");
        std::process::exit(1);
    };

    let table: [(&str, &[&str]); 26] = [
        ("a", &["a", "aa", "e", "oi", "o", "nya", "y"]),
        ("b", &["b", "bh"]),
        ("c", &["c", "ch", "k"]),
        ("d", &["d", "dh", "dd", "ddh"]),
        ("e", &["i", "ii", "e", "y"]),
        ("f", &["ph"]),
        ("g", &["g", "gh", "j"]),
        ("h", &["h"]),
        ("i", &["i", "ii", "y"]),
        ("j", &["j", "jh", "z"]),
        ("k", &["k", "kh"]),
        ("l", &["l"]),
        ("m", &["h", "m"]),
        ("n", &["n", "nya", "nga", "nn"]),
        ("o", &["a", "u", "uu", "oi", "o", "ou", "y"]),
        ("p", &["p", "ph"]),
        ("q", &["k"]),
        ("r", &["rri", "h", "r", "rr", "rrh"]),
        ("s", &["s", "sh", "ss"]),
        ("t", &["t", "th", "tt", "tth", "khandatta"]),
        ("u", &["u", "uu", "y"]),
        ("v", &["bh"]),
        ("w", &["o"]),
        ("x", &["e", "k"]),
        ("y", &["i", "y"]),
        ("z", &["h", "j", "jh", "z"]),
    ];
    let table: HashMap<&'static str, &'static [&'static str], RandomState> =
        table.into_iter().collect();
    let database: HashMap<String, Vec<String>, RandomState> =
        serde_json::from_slice(include_bytes!("../data/dictionary.json")).unwrap();
    let builder = Parser::new_regex();
    let mut regex = String::with_capacity(1024);

    let mut suggest = |input: &str| -> Vec<String> {
        builder.convert_regex_into(input, &mut regex);
        let rgx = Regex::new(&regex).unwrap();

        table
            .get(input.get(0..1).unwrap_or_default())
            .copied()
            .unwrap_or_default()
            .iter()
            .flat_map(|&item| {
                database
                    .get(item)
                    .unwrap()
                    .iter()
                    .filter(|i| rgx.is_match(i))
            })
            .cloned()
            .collect()
    };

    let mut suggestions = suggest(&word);

    println!("Word: {}", word);
    suggestions.sort();
    println!("Suggestions: [{}]", suggestions.join(", "));

    let current_mem = PEAK_ALLOC.current_usage_as_mb();
    println!("This program currently uses {} MB of RAM.", current_mem);
    let peak_mem = PEAK_ALLOC.peak_usage_as_mb();
    println!("The max amount that was used {} MB.", peak_mem);
}
