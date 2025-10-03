use std::{collections::HashMap, hint::black_box};

use ahash::RandomState;
use criterion::{criterion_group, criterion_main, Criterion};
use okkhor::parser::Parser;
use regex::Regex;

use upodesh::avro::Suggest;

fn upodesh_avro_benchmark(c: &mut Criterion) {
    let suggest = Suggest::new();

    c.bench_function("upodesh avro a", |b| b.iter(|| suggest.suggest(black_box("a"))));
    c.bench_function("upodesh avro arO", |b| {
        b.iter(|| suggest.suggest(black_box("arO")))
    });
    c.bench_function("upodesh avro bistari", |b| {
        b.iter(|| suggest.suggest(black_box("bistari")))
    });
}

fn regex_avro_benchmark(c: &mut Criterion) {
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

    c.bench_function("regex avro a", |b| b.iter(|| suggest(black_box("a"))));
    c.bench_function("regex avro arO", |b| b.iter(|| suggest(black_box("arO"))));
    c.bench_function("regex avro bistari", |b| {
        b.iter(|| suggest(black_box("bistari")))
    });
}

fn upodesh_bangla_benchmark(c: &mut Criterion) {
    use upodesh::bangla::suggest;
    c.bench_function("upodesh bangla আমা", |b| {
        b.iter(|| suggest(black_box("আমা")))
    });
    c.bench_function("upodesh bangla কম্পি", |b| {
        b.iter(|| suggest(black_box("কম্পি")))
    });
    c.bench_function("upodesh bangla কনট্রো", |b| {
        b.iter(|| suggest(black_box("কনট্রো")))
    });
}

fn regex_bangla_benchmark(c: &mut Criterion) {
    fn suggest(word: &str) -> Vec<String> {
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

    c.bench_function("regex bangla আমা", |b| {
        b.iter(|| suggest(black_box("আমা")))
    });
    c.bench_function("regex bangla কম্পি", |b| {
        b.iter(|| suggest(black_box("কম্পি")))
    });
    c.bench_function("regex bangla কনট্রো", |b| {
        b.iter(|| suggest(black_box("কনট্রো")))
    });
}

criterion_group!(benches_avro, upodesh_avro_benchmark, regex_avro_benchmark);
criterion_group!(
    benches_bangla,
    upodesh_bangla_benchmark,
    regex_bangla_benchmark
);
criterion_main!(benches_avro, benches_bangla);
