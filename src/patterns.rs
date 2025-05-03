use std::collections::HashMap;

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
        let patterns = include_str!("../data/preprocessed-patterns.json");
        let common_data = include_str!("../data/source-common-patterns.json");

        let dict: HashMap<String, Block> = serde_json::from_str(patterns).unwrap();
        let common = serde_json::from_str(common_data).unwrap();

        let trie = Trie::from_strings(dict.keys().map(|s| s.as_str()));
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
