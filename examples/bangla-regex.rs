use std::collections::HashMap;

use ahash::RandomState;
use regex::Regex;

pub(crate) fn search_dictionary(word: &str) -> Vec<String> {
    let table = match word.chars().next().unwrap_or_default() {
        // Kars
        'া' => "aa",
        'ি' => "i",
        'ী' => "ii",
        'ু' => "u",
        'ূ' => "uu",
        'ৃ' => "rri",
        'ে' => "e",
        'ৈ' => "oi",
        'ো' => "o",
        'ৌ' => "ou",
        // Vowels
        'অ' => "a",
        'আ' => "aa",
        'ই' => "i",
        'ঈ' => "ii",
        'উ' => "u",
        'ঊ' => "uu",
        'ঋ' => "rri",
        'এ' => "e",
        'ঐ' => "oi",
        'ও' => "o",
        'ঔ' => "ou",
        // Consonants
        'ক' => "k",
        'খ' => "kh",
        'গ' => "g",
        'ঘ' => "gh",
        'ঙ' => "nga",
        'চ' => "c",
        'ছ' => "ch",
        'জ' => "j",
        'ঝ' => "jh",
        'ঞ' => "nya",
        'ট' => "tt",
        'ঠ' => "tth",
        'ড' => "dd",
        'ঢ' => "ddh",
        'ণ' => "nn",
        'ত' => "t",
        'থ' => "th",
        'দ' => "d",
        'ধ' => "dh",
        'ন' => "n",
        'প' => "p",
        'ফ' => "ph",
        'ব' => "b",
        'ভ' => "bh",
        'ম' => "m",
        'য' => "z",
        'র' => "r",
        'ল' => "l",
        'শ' => "sh",
        'ষ' => "ss",
        'স' => "s",
        'হ' => "h",
        'ড়' => "rr",
        'ঢ়' => "rrh",
        'য়' => "y",
        'ৎ' => "khandatta",
        // Otherwise we don't have any suggestions to search from, so return from the function.
        _ => return Vec::new(),
    };

    let word = clean_string(word);

    let need_chars_upto = match word.chars().count() {
        1 => 0,
        2..=3 => 1,
        _ => 5,
    };

    let regex = format!(
        "^{word}[অআইঈউঊঋএঐওঔঌৡািীুূৃেৈোৌকখগঘঙচছজঝঞটঠডঢণতথদধনপফবভমযরলশষসহৎড়ঢ়য়ংঃঁ\u{09CD}]{{0,{need_chars_upto}}}$"
    );
    let rgx = Regex::new(&regex).unwrap();

    let database: HashMap<String, Vec<String>, RandomState> =
        serde_json::from_slice(include_bytes!("../data/dictionary.json")).unwrap();

    database
        .get(table)
        .unwrap()
        .into_iter()
        .filter(|i| rgx.is_match(i))
        .cloned()
        .collect()
}

fn clean_string(string: &str) -> String {
    string
        .chars()
        .filter(|&c| !"|()[]{}^$*+?.~!@#%&-_='\";<>/\\,:`।\u{200C}".contains(c))
        .collect()
}

fn main() {
    let Some(word) = std::env::args().nth(1) else {
        eprintln!("Please provide a word");
        std::process::exit(1);
    };

    let mut suggestions = search_dictionary(&word);

    println!("Word: {}", word);
    suggestions.sort();
    println!("Suggestions: [{}]", suggestions.join(", "));
}
