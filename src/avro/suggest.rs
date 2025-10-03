use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{fst::FstTree, avro::utils::fix_string, WORDS};

static PATTERNS: Lazy<FstTree<&[u8]>> =
    Lazy::new(|| FstTree::from_fst(include_bytes!("patterns.fst")));

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transliterate: Vec<String>,
    pub entire_block_optional: Option<bool>,
}

pub struct Suggest {
    patterns: HashMap<String, Block>,
    common_suffixes: Vec<String>,
}

impl Suggest {
    pub fn new() -> Self {
        let patterns_data = include_bytes!("../../data/preprocessed-patterns.json");
        let common_data = include_bytes!("../../data/source-common-patterns.json");

        let patterns: HashMap<String, Block> = serde_json::from_slice(patterns_data).unwrap();
        let common_suffixes = serde_json::from_slice(common_data).unwrap();

        Suggest {
            patterns,
            common_suffixes,
        }
    }

    pub fn suggest(&self, input: &str) -> Vec<String> {
        let words = Lazy::force(&WORDS);
        let patterns = Lazy::force(&PATTERNS);
        let input = fix_string(input);

        let (matched, mut remaining, _) = patterns.match_longest_common_prefix(&input);

        let matched_patterns = if let Some(block) = self.patterns.get(matched) {
            &block.transliterate
        } else {
            return vec![];
        };

        let mut matched_nodes = matched_patterns
            .iter()
            .filter_map(|p| words.matching_node(p))
            .collect::<Vec<_>>();

        let additional_nodes = matched_nodes
            .iter()
            .flat_map(|node| {
                self.common_suffixes
                    .iter()
                    .filter_map(|suffix| node.get_matching_node(suffix))
            })
            .collect::<Vec<_>>();

        matched_nodes.extend(additional_nodes);

        while !remaining.is_empty() {
            let (mut new_matched, new_remaining, mut complete) =
                patterns.match_longest_common_prefix(remaining);

            if !complete {
                for i in (0..remaining.len()).rev() {
                    (new_matched, _, complete) =
                        patterns.match_longest_common_prefix(&remaining[..i]);

                    if complete {
                        remaining = &remaining[i..];
                        break;
                    }
                }
            } else {
                remaining = new_remaining;
            }

            let new_matched_patterns = if let Some(block) = self.patterns.get(new_matched) {
                &block.transliterate
            } else {
                // If no patterns match, we can stop here
                break;
            };

            let new_matched_nodes = new_matched_patterns
                .iter()
                .flat_map(|p| {
                    matched_nodes
                        .iter()
                        .filter_map(|node| node.get_matching_node(p))
                })
                .collect::<Vec<_>>();

            if self
                .patterns
                .get(new_matched)
                .map_or(false, |v| v.entire_block_optional.is_some())
            {
                // Entirely optional patterns like "([ওোঅ]|(অ্য)|(য়ো?))?" may not yield any result
                matched_nodes.extend(new_matched_nodes);
            } else {
                matched_nodes = new_matched_nodes;
            }

            let additional_matched_nodes = matched_nodes
                .iter()
                .flat_map(|node| {
                    self.common_suffixes
                        .iter()
                        .filter_map(|suffix| node.get_matching_node(suffix))
                })
                .collect::<Vec<_>>();
            matched_nodes.extend(additional_matched_nodes);
        }

        let suggestions: HashSet<_> = matched_nodes
            .into_iter()
            .filter_map(|n| n.get_word())
            .collect();
        suggestions.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sort(mut vec: Vec<String>) -> Vec<String> {
        vec.sort();
        vec
    }

    #[test]
    fn test_suggestions() {
        let suggest = Suggest::new();

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
        assert_eq!(
            sort(suggest.suggest("cool")),
            vec!["চুল", "চূল", "চোল", "ছুঁল", "ছুল", "ছোল"]
        );
        assert_eq!(
            sort(suggest.suggest("shokti")),
            vec!["শকতি", "শক্তি", "সক্তি"]
        );
        assert_eq!(sort(suggest.suggest("chup")), vec!["চুপ", "ছুপ"]);
        assert_eq!(
            sort(suggest.suggest("as")),
            vec!["অশ্ব", "অশ্ম", "আঁশ", "আশ", "আস", "এস"]
        );
        assert_eq!(sort(suggest.suggest("apni")), vec!["আপনি"]);
        assert_eq!(
            sort(suggest.suggest("kkhet")),
            vec!["ক্ষেত", "খেঁট", "খেট", "খেত", "খ্যাঁট", "খ্যাঁত", "খ্যাত"]
        );
        assert_eq!(sort(suggest.suggest("ebong")), vec!["এবং"]);
        assert_eq!(sort(suggest.suggest("shesh")), vec!["শেষ", "সেস"]);
    }

    #[test]
    fn test_empty_suggestion() {
        let suggest = Suggest::new();

        assert_eq!(suggest.suggest("6t``"), Vec::<String>::new());
        assert_eq!(suggest.suggest("6t`"), Vec::<String>::new());
        assert_eq!(suggest.suggest("t6th"), Vec::<String>::new());
    }
}
