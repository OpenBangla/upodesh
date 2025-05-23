use std::collections::HashMap;
use std::hint::black_box;

use ahash::RandomState;
use criterion::{Criterion, criterion_group, criterion_main};
use okkhor::parser::Parser;
use regex::Regex;

use upodesh::{regex::RegexSuggest, suggest::Suggest};

fn upodesh_benchmark(c: &mut Criterion) {
    let suggest = Suggest::new();

    c.bench_function("upodesh a", |b| b.iter(|| suggest.suggest(black_box("a"))));
    c.bench_function("upodesh arO", |b| {
        b.iter(|| suggest.suggest(black_box("arO")))
    });
    c.bench_function("upodesh bistari", |b| {
        b.iter(|| suggest.suggest(black_box("bistari")))
    });
}

fn upodesh_fst_benchmark(c: &mut Criterion) {
    let mut suggest = RegexSuggest::new();

    c.bench_function("upodesh-fst a", |b| b.iter(|| suggest.suggest(black_box("a"))));
    c.bench_function("upodesh-fst arO", |b| {
        b.iter(|| suggest.suggest(black_box("arO")))
    });
    c.bench_function("upodesh-fst bistari", |b| {
        b.iter(|| suggest.suggest(black_box("bistari")))
    });
}

fn regex_benchmark(c: &mut Criterion) {
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

    c.bench_function("regex a", |b| b.iter(|| suggest(black_box("a"))));
    c.bench_function("regex arO", |b| b.iter(|| suggest(black_box("arO"))));
    c.bench_function("regex bistari", |b| {
        b.iter(|| suggest(black_box("bistari")))
    });
}

criterion_group!(benches, upodesh_benchmark, upodesh_fst_benchmark, regex_benchmark);
criterion_main!(benches);
