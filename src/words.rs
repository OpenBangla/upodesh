use crate::trie::Trie;

pub struct Words {
    pub trie: Trie,
}

impl Words {
    pub fn new() -> Self {
        let words = include_str!("../data/source-words.txt");

        let trie = Trie::from_strings(words.lines().map(|s| s.trim()));
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
