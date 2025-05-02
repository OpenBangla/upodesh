use crate::trie::Trie;

pub struct Words {
    pub trie: Trie,
}

impl Words {
    pub fn new() -> Self {
        let words = include_str!("../data/source-words.txt");
        
        let mut trie = Trie::new();
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
