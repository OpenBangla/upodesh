use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{fst::FstTree, WORDS};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Block {
    pub transliterate: Vec<String>,
    pub entire_block_optional: Option<bool>,
}

/// Suggestion maker for the transliterative methods, driven by a table of
/// roman patterns to their possible Bangla alternatives.
pub(crate) struct PatternSuggest {
    patterns_fst: FstTree<&'static [u8]>,
    patterns: HashMap<String, Block>,
    common_suffixes: Vec<&'static str>,
}

impl PatternSuggest {
    pub fn new(
        patterns_fst: &'static [u8],
        patterns_data: &'static [u8],
        common_data: &'static str,
    ) -> Self {
        let patterns: HashMap<String, Block> = serde_json::from_slice(patterns_data).unwrap();
        let common_suffixes = common_data.lines().collect();

        PatternSuggest {
            patterns_fst: FstTree::from_fst(patterns_fst),
            patterns,
            common_suffixes,
        }
    }

    /// Find the longest pattern which is a prefix of `input`.
    ///
    /// Returns the matched pattern's block and the remaining input.
    fn match_pattern<'a>(&self, input: &'a str) -> Option<(&Block, &'a str)> {
        let (mut matched, mut remaining, complete) =
            self.patterns_fst.match_longest_common_prefix(input);

        if !complete {
            // The longest common prefix is only a part of a longer pattern,
            // so look for the longest prefix which is a pattern itself.
            let (prefix, rest) = (0..matched.len())
                .rev()
                .map(|i| input.split_at(i))
                .find(|(prefix, _)| self.patterns_fst.match_longest_common_prefix(prefix).2)?;

            (matched, remaining) = (prefix, rest);
        }

        self.patterns.get(matched).map(|block| (block, remaining))
    }

    /// Make suggestions for an `input` which is already normalized by the method.
    pub fn suggest(&self, input: &str) -> Vec<String> {
        let words = Lazy::force(&WORDS);

        let (block, mut remaining) = if let Some(m) = self.match_pattern(input) {
            m
        } else {
            return vec![];
        };

        let mut matched_nodes = block
            .transliterate
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
            let block = if let Some((block, new_remaining)) = self.match_pattern(remaining) {
                remaining = new_remaining;
                block
            } else {
                // If no patterns match, we can stop here
                break;
            };

            let new_matched_nodes = block
                .transliterate
                .iter()
                .flat_map(|p| {
                    matched_nodes
                        .iter()
                        .filter_map(|node| node.get_matching_node(p))
                })
                .collect::<Vec<_>>();

            if block.entire_block_optional.is_some() {
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
