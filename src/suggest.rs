use std::collections::HashSet;

use crate::{patterns::Patterns, trie::TrieNode, utils::fix_string, words::Words};

pub struct Suggest {
    patterns: Patterns,
    words: Words,
}

impl Suggest {
    pub fn new() -> Self {
        let patterns = Patterns::new();
        let words = Words::new();

        Suggest { patterns, words }
    }

    pub fn suggest(&self, input: &str) -> Vec<String> {
        let input = fix_string(input);

        let (matched, mut remaining, _) = self.patterns.trie.match_longest_common_prefix(&input);

        let matched_patterns = &self.patterns.dict.get(matched).unwrap().transliterate;
        let common_patterns_len = self.patterns.common.len();
        let mut matched_nodes: Vec<&TrieNode> =
            Vec::with_capacity(matched_patterns.len() * common_patterns_len);

        for p in matched_patterns {
            if let Some(node) = self.words.trie.matching_node(p) {
                matched_nodes.push(node);
            }

            // Try matching optional patterns too
            let mut additional_nodes: Vec<&TrieNode> =
                Vec::with_capacity(matched_nodes.len() * common_patterns_len);

            for matched_node in matched_nodes.iter() {
                for common in self.patterns.common.iter() {
                    if let Some(node) = matched_node.get_matching_node(common) {
                        additional_nodes.push(node);
                    }
                }
            }

            // Merge additional nodes with matched_nodes
            matched_nodes.extend(additional_nodes);
        }

        while !remaining.is_empty() {
            let (mut new_matched, mut new_remaining, mut complete) =
                self.patterns.trie.match_longest_common_prefix(&remaining);

            if !complete {
                for i in (0..remaining.len()).rev() {
                    (new_matched, new_remaining, complete) = self
                        .patterns
                        .trie
                        .match_longest_common_prefix(&remaining[..i]);

                    if complete {
                        remaining = &remaining[i..];
                        break;
                    }
                }
            } else {
                remaining = new_remaining;
            }

            let new_matched_patterns = &self.patterns.dict.get(new_matched).unwrap().transliterate;
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
                .dict
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
                for common in self.patterns.common.iter() {
                    if let Some(node) = matched_node.get_matching_node(common) {
                        additional_nodes.push(node);
                    }
                }
            }

            // Merge additional nodes with matched_nodes
            matched_nodes.extend(additional_nodes);
        }

        let suggestions: HashSet<String> =
            matched_nodes.iter().filter_map(|n| n.get_word()).collect();
        suggestions.into_iter().collect()
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
    }
}
