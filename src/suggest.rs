use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::{
    trie::{Trie, TrieNode},
    utils::fix_string,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transliterate: Vec<String>,
    pub entire_block_optional: Option<bool>,
}

pub struct Suggest {
    patterns: HashMap<String, Block>,
    patterns_trie: Trie,
    words: Trie,
    common_suffixes: Vec<String>,
}

impl Suggest {
    pub fn new() -> Self {
        let patterns_data = include_bytes!("../data/preprocessed-patterns.json");
        let words_data = include_str!("../data/source-words.txt");
        let common_data = include_bytes!("../data/source-common-patterns.json");

        let patterns: HashMap<String, Block> = serde_json::from_slice(patterns_data).unwrap();
        let patterns_trie = Trie::from_strings(patterns.keys().map(|s| s.as_str()));
        let words = Trie::from_strings(words_data.lines().map(|s| s.trim()));
        let common_suffixes = serde_json::from_slice(common_data).unwrap();

        Suggest {
            patterns,
            patterns_trie,
            words,
            common_suffixes,
        }
    }

    pub fn suggest(&self, input: &str) -> Vec<String> {
        let input = fix_string(input);

        let (matched, mut remaining, _) = self.patterns_trie.match_longest_common_prefix(&input);

        let matched_patterns = &self.patterns.get(matched).unwrap().transliterate;
        let common_patterns_len = self.common_suffixes.len();
        let mut matched_nodes: Vec<&TrieNode> =
            Vec::with_capacity(matched_patterns.len() * common_patterns_len);

        for p in matched_patterns {
            if let Some(node) = self.words.matching_node(p) {
                matched_nodes.push(node);
            }

            // Try matching optional patterns too
            let mut additional_nodes: Vec<&TrieNode> =
                Vec::with_capacity(matched_nodes.len() * common_patterns_len);

            for matched_node in matched_nodes.iter() {
                for suffix in self.common_suffixes.iter() {
                    if let Some(node) = matched_node.get_matching_node(suffix) {
                        additional_nodes.push(node);
                    }
                }
            }

            // Merge additional nodes with matched_nodes
            matched_nodes.extend(additional_nodes);
        }

        while !remaining.is_empty() {
            let (mut new_matched, mut new_remaining, mut complete) =
                self.patterns_trie.match_longest_common_prefix(&remaining);

            if !complete {
                for i in (0..remaining.len()).rev() {
                    (new_matched, new_remaining, complete) = self
                        .patterns_trie
                        .match_longest_common_prefix(&remaining[..i]);

                    if complete {
                        remaining = &remaining[i..];
                        break;
                    }
                }
            } else {
                remaining = new_remaining;
            }

            let new_matched_patterns = &self.patterns.get(new_matched).unwrap().transliterate;
            let mut new_matched_nodes: Vec<&TrieNode> =
                Vec::with_capacity(new_matched_patterns.len());

            for p in new_matched_patterns {
                for node in matched_nodes.iter() {
                    if let Some(new_node) = node.get_matching_node(p) {
                        new_matched_nodes.push(new_node);
                    }
                }
            }

            if self
                .patterns
                .get(new_matched)
                .unwrap()
                .entire_block_optional
                .is_some()
            {
                // Entirely optional patterns like "([ওোঅ]|(অ্য)|(য়ো?))?" may not yield any result
                matched_nodes.extend(new_matched_nodes);
            } else {
                matched_nodes = new_matched_nodes;
            }

            // Try matching optional patterns too
            let mut additional_nodes: Vec<&TrieNode> =
                Vec::with_capacity(matched_nodes.len() * common_patterns_len);

            for matched_node in matched_nodes.iter() {
                for suffix in self.common_suffixes.iter() {
                    if let Some(node) = matched_node.get_matching_node(suffix) {
                        additional_nodes.push(node);
                    }
                }
            }

            // Merge additional nodes with matched_nodes
            matched_nodes.extend(additional_nodes);
        }

        let suggestions: HashSet<_> = matched_nodes.iter().filter_map(|n| n.get_word()).collect();
        suggestions.into_iter().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestions() {
        let suggest = Suggest::new();

        fn sort(mut vec: Vec<String>) -> Vec<String> {
            vec.sort();
            vec
        }

        assert_eq!(
            sort(suggest.suggest("sari")),
            vec![
                "শারি",
                "শারী",
                "শাড়ি",
                "শাড়ী",
                "সারি",
                "সারী",
                "সাড়ি",
                "সাড়ী",
                "স্মঅরী"
            ]
        );
        assert_eq!(sort(suggest.suggest("sar")), vec!["ষাঁড়", "সার", "সার্ব", "সাড়"]);
        assert_eq!(sort(suggest.suggest("amra")), vec!["অমরা", "আমরা", "আমড়া"]);
        assert_eq!(sort(suggest.suggest("lalshak")), vec!["লালশাক"]);
        assert_eq!(sort(suggest.suggest("lalrong")), vec!["লালরং", "লালরঙ"]);
        assert_eq!(sort(suggest.suggest("ongshochched")), vec!["অংশচ্ছেদ"]);
        assert_eq!(sort(suggest.suggest("ongshocched")), vec!["অংশচ্ছেদ"]);
        assert_eq!(sort(suggest.suggest("shadhinota")), vec!["স্বাধীনতা"]);
        assert_eq!(sort(suggest.suggest("dukkho")), vec!["দুঃখ", "দুখ"]);
        assert_eq!(sort(suggest.suggest("cool")), vec!["চুল", "চূল", "চোল", "ছুঁল", "ছুল", "ছোল"]);
        assert_eq!(sort(suggest.suggest("shokti")), vec!["শকতি", "শক্তি", "সক্তি"]);
        assert_eq!(sort(suggest.suggest("chup")), vec!["চুপ", "ছুপ"]);
        assert_eq!(sort(suggest.suggest("as")), vec!["অশ্ব", "অশ্ম", "আঁশ", "আশ", "আস", "এস"]);
        assert_eq!(sort(suggest.suggest("apni")), vec!["আপনি"]);
        assert_eq!(sort(suggest.suggest("kkhet")), vec!["ক্ষেত", "খেঁট", "খেট", "খেত", "খ্যাঁট", "খ্যাঁত", "খ্যাত"]);
        assert_eq!(sort(suggest.suggest("ebong")), vec!["এবং"]);
        assert_eq!(sort(suggest.suggest("shesh")), vec!["শেষ", "সেস"]);
    }
}
