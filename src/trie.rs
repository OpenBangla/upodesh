use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TrieNode {
    children: HashMap<char, TrieNode>,
    word: Option<String>, // Store the complete word at the end of the node
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            word: None,
        }
    }

    fn is_complete_word(&self) -> bool {
        self.word.is_some()
    }

    pub fn get_word(&self) -> Option<String> {
        self.word.clone()
    }

    fn find_complete_words(&self) -> Vec<String> {
        let mut words = Vec::new();
        for node in self.children.values() {
            if let Some(word) = &node.word {
                words.push(word.clone());
            }

            let child_words = node.find_complete_words();
            words.extend(child_words);
        }

        words
    }

    /// FindMatchingNode
    pub fn get_matching_node(&self, word: &str) -> Option<&TrieNode> {
        let mut current_node = self;
        for ch in word.chars() {
            match current_node.children.get(&ch) {
                Some(node) => current_node = node,
                None => return None,
            }
        }
        Some(current_node)
    }
}

pub struct Trie {
    root: TrieNode,
}

impl Trie {
    fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    fn insert(&mut self, word: &str) {
        let mut current_node = &mut self.root;
        for ch in word.chars() {
            current_node = current_node.children.entry(ch).or_insert(TrieNode::new());
        }

        current_node.word = Some(word.to_string()); // Store the word at the end of the node
    }

    pub fn from_strings<'a>(words: impl Iterator<Item = &'a str>) -> Self {
        let mut trie = Trie::new();
        for word in words {
            trie.insert(word);
        }
        trie
    }

    /// FindMatchingNode
    pub fn matching_node(&self, word: &str) -> Option<&TrieNode> {
        let mut current_node = &self.root;
        for ch in word.chars() {
            match current_node.children.get(&ch) {
                Some(node) => current_node = node,
                None => return None,
            }
        }
        Some(current_node)
    }

    /// findLongestPrefixNode
    fn longest_prefix(&self, word: &str) -> (&TrieNode, usize) {
        let mut current_node = &self.root;
        let mut prefix_len: usize = 0;

        for ch in word.chars() {
            match current_node.children.get(&ch) {
                Some(node) => {
                    prefix_len += ch.len_utf8();
                    current_node = node;
                }
                None => break,
            }
        }

        (current_node, prefix_len)
    }

    /// MatchPrefix
    pub fn match_prefix(&self, prefix: &str) -> Vec<String> {
        let mut result = Vec::new();

        if prefix.is_empty() {
            return result;
        }

        let (node, prefix_len) = self.longest_prefix(prefix);
        if prefix_len == 0 {
            return result;
        };

        result.extend(node.find_complete_words());

        if node.word.is_some() {
            result.push(node.word.clone().unwrap());
        }

        result
    }

    /// MatchLongestCommonPrefix
    pub fn match_longest_common_prefix<'a>(&self, prefix: &'a str) -> (&'a str, &'a str, bool) {
        if prefix.is_empty() {
            return ("", "", false);
        }

        let (node, matched_prefix_len) = self.longest_prefix(prefix);
        let (matched_prefix, remaining) = prefix.split_at(matched_prefix_len);
        (matched_prefix, remaining, node.word.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_node() {
        let trie = Trie::from_strings(["ক", "কখ", "কখগঘঙচছ"].into_iter());

        let n1 = trie.matching_node("ক").unwrap();

        let n2 = n1.get_matching_node("খ").unwrap();

        _ = n2.get_matching_node("গঘ").unwrap();

        _ = trie.matching_node("কখগঘ").unwrap();
    }

    #[test]
    fn test_is_complete_word() {
        let trie = Trie::from_strings(["ক", "কখ", "কখগঘঙচছ"].into_iter());

        let n1 = trie.matching_node("ক").unwrap();
        assert!(n1.is_complete_word());

        let n2 = n1.get_matching_node("খ").unwrap();
        assert!(n2.is_complete_word());

        let n3 = n2.get_matching_node("গঘ").unwrap();
        assert!(!n3.is_complete_word());

        let n4 = trie.matching_node("কখগঘ").unwrap();
        assert!(!n4.is_complete_word());
    }

    #[test]
    fn test_match_prefix() {
        let trie = Trie::from_strings(["ক", "কখগ", "কখগঘঙ", "চ", "চছজ", "চছজঝঞ", "১"].into_iter());

        assert_eq!(trie.match_prefix("ক"), vec!["কখগ", "কখগঘঙ", "ক"]);
        assert_eq!(trie.match_prefix("কখ"), vec!["কখগ", "কখগঘঙ"]);
        assert_eq!(trie.match_prefix("চছজঝঞ"), vec!["চছজঝঞ"]);
        assert_eq!(trie.match_prefix("২"), Vec::<String>::new());
        assert_eq!(trie.match_prefix(""), Vec::<String>::new());
    }

    #[test]
    fn test_match_longest_common_prefix() {
        let trie = Trie::from_strings(["ক", "কখগ", "কখগঘঙ", "চ", "চছজ", "চছজঝঞ", "১"].into_iter());

        assert_eq!(trie.match_longest_common_prefix("ক"), ("ক", "", true));
        assert_eq!(
            trie.match_longest_common_prefix("ক1234"),
            ("ক", "1234", true)
        );
        assert_eq!(
            trie.match_longest_common_prefix("1234"),
            ("", "1234", false)
        );
        assert_eq!(
            trie.match_longest_common_prefix("কখগঘঙচছজঝঞ"),
            ("কখগঘঙ", "চছজঝঞ", true)
        );
    }
}
