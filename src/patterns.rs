use std::{collections::HashMap, fs::read_to_string};

use serde::Deserialize;

use crate::trie::Trie;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transliterate: Vec<String>,
    pub entire_block_optional: Option<bool>,
}

pub struct Patterns {
    pub dict: HashMap<String, Block>,
    pub trie: Trie,
    pub common: Vec<String>,
}

impl Patterns {
    pub fn new() -> Self {
        let patterns = read_to_string("./data/preprocessed-patterns.json").unwrap();
        let common_data = read_to_string("./data/source-common-patterns.json").unwrap();

        let common = serde_json::from_str(&common_data).unwrap();

        let dict: HashMap<String, Block> = serde_json::from_str(&patterns).unwrap();
        let mut trie = Trie::new();

        for key in dict.keys() {
            trie.insert(key);
        }

        Patterns { dict, trie, common }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns_loading() {
        let patterns = Patterns::new();

        let optional_block = patterns.dict.get("o").unwrap();
        assert!(optional_block.entire_block_optional.is_some());
    }
}
