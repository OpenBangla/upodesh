use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{fst::FstTree, WORDS};

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
    common_suffixes: Vec<&'static str>,
}

impl Suggest {
    pub fn new() -> Self {
        let patterns_data = include_bytes!("../../data/khipro-preprocessed-patterns.json");
        let common_data = include_str!("../../data/source-common-patterns.txt");

        let patterns: HashMap<String, Block> = serde_json::from_slice(patterns_data).unwrap();
        let common_suffixes = common_data.lines().collect();

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

fn fix_string(s: &str) -> String {
    s.trim().to_ascii_lowercase()
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

        assert_eq!(sort(suggest.suggest("amar")), vec!["আমার"]);
        assert_eq!(sort(suggest.suggest("kaka")), vec!["কাকা"]);
        assert_eq!(sort(suggest.suggest("bangla")), vec!["বাংলা"]);
        assert_eq!(sort(suggest.suggest("khata")), vec!["খাতা"]);
        assert_eq!(sort(suggest.suggest("ami")), vec!["আমি"]);
        assert_eq!(sort(suggest.suggest("ebong")), vec!["এবং"]);
        assert_eq!(sort(suggest.suggest("zokhon")), vec!["যখন"]);
        assert_eq!(sort(suggest.suggest("garfi")), vec!["গা\u{09DC}ি"]);
        assert_eq!(sort(suggest.suggest("phol")), vec!["ফল"]);
    }

    #[test]
    fn test_conjuncts() {
        let suggest = Suggest::new();

        assert_eq!(sort(suggest.suggest("shokti")), vec!["শক্তি"]);
        assert_eq!(sort(suggest.suggest("biggan")), vec!["বিজ্ঞান"]);
        assert_eq!(sort(suggest.suggest("ggan")), vec!["জ্ঞান"]);
        assert_eq!(sort(suggest.suggest("cottfgram")), vec!["চট্টগ্রাম"]);
        assert_eq!(sort(suggest.suggest("shikfa")), vec!["শিক্ষা"]);
        assert_eq!(sort(suggest.suggest("poncash")), vec!["পঞ্চাশ"]);
        assert_eq!(sort(suggest.suggest("oncol")), vec!["অঞ্চল"]);
        assert_eq!(sort(suggest.suggest("gorrb")), vec!["গর্ব"]);
        assert_eq!(sort(suggest.suggest("porrzay")), vec!["পর্যা\u{09DF}"]);
        assert_eq!(sort(suggest.suggest("sbadhiinota")), vec!["স্বাধীনতা"]);
    }

    #[test]
    fn test_empty_suggestion() {
        let suggest = Suggest::new();

        assert_eq!(suggest.suggest("zzzzz"), Vec::<String>::new());
    }
}
