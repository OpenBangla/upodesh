use std::fs::read_to_string;

use crate::trie::Trie;

pub struct Words {
    pub trie: Trie,
}

impl Words {
    pub fn new() -> Self {
        let mut trie = Trie::new();
        let words = read_to_string("./data/source-words.txt").unwrap();

        for word in words.lines() {
            trie.insert(word.trim());
        }

        Words { trie }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_words_load() {
        _ = Words::new();
    }
}
