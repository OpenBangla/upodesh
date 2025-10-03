use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env::{args, var_os},
    fs::{File, read, read_to_string},
    io::BufWriter,
    path::PathBuf,
};

use fst::raw::Builder;
use rexplode::explode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transliterate: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entire_block_optional: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegexBlock {
    pub transliterate: String,
    pub entire_block_optional: Option<bool>,
}

fn generate_words_fst() {
    let root = PathBuf::from(var_os("CARGO_MANIFEST_DIR").unwrap());
    let parent = root.parent().unwrap();
    let dest = parent.join("src").join("words.fst");

    let file = File::create(dest).expect("Failed to create words.fst");
    let writer = BufWriter::new(file);

    let mut fst = Builder::new(writer).unwrap();
    let words = read_to_string(parent.join("data/source-words.txt"))
        .expect("Failed to read source words file");

    let mut words = words.lines().map(str::trim).collect::<Vec<_>>();

    words.sort();

    for word in words {
        let numbered_word = word.chars().map(|c| c as u8).collect::<Vec<u8>>();
        fst.add(&numbered_word).expect("Failed to add word to FST");
    }

    fst.finish().expect("Failed to finish words FST generation");
}

fn generate_patterns_fst() {
    let root = PathBuf::from(var_os("CARGO_MANIFEST_DIR").unwrap());
    let parent = root.parent().unwrap();
    let dest = parent.join("src").join("avro").join("patterns.fst");

    let file = File::create(dest).expect("Failed to create patterns.fst");
    let writer = BufWriter::new(file);

    let mut fst = Builder::new(writer).unwrap();
    let patterns: HashMap<String, Block> = serde_json::from_slice(
        &read(parent.join("data/preprocessed-patterns.json"))
            .expect("Failed to read source patterns file"),
    )
    .unwrap();

    let mut patterns = patterns.keys().map(|s| s.as_str()).collect::<Vec<_>>();

    patterns.sort();

    for pattern in patterns {
        let numbered_word = pattern.chars().map(|c| c as u8).collect::<Vec<u8>>();
        fst.add(&numbered_word)
            .expect("Failed to add pattern to FST");
    }

    fst.finish()
        .expect("Failed to finish patterns FST generation");
}

fn generate_regex_exploded_patterns(source: &str, dest: &str) {
    let file = File::create(dest).expect("Failed to create destination file");

    let regex_patterns: HashMap<String, RegexBlock> =
        serde_json::from_slice(&read(source).expect("Failed to read source patterns file"))
            .unwrap();

    let mut patterns: BTreeMap<String, Block> = BTreeMap::new();

    for (pattern, block) in regex_patterns {
        // Unique and non-empty patterns only
        let mut exploded: Vec<String> = explode(&block.transliterate)
            .unwrap()
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

        // sort the patterns for consistency
        exploded.sort();

        patterns.insert(
            pattern,
            Block {
                transliterate: exploded,
                entire_block_optional: block.entire_block_optional,
            },
        );
    }

    serde_json::to_writer_pretty(file, &patterns)
        .expect("Failed to write exploded patterns to file");
}

fn main() {
    let args = args().collect::<Vec<String>>();

    if let Some("explode") = args.get(1).map(|s| s.as_str()) {
        let source = args
            .get(2)
            .map(|s| s.as_str())
            .expect("Give source regex pattern json");
        let dest = args
            .get(3)
            .map(|s| s.as_str())
            .expect("Give destination path");

        generate_regex_exploded_patterns(source, dest);
    } else {
        generate_words_fst();
        generate_patterns_fst();
    }
}
