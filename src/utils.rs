use louds_rs::LoudsNodeNum;
use trie_rs::{inc_search::{IncSearch, Position}, Trie};

pub fn fix_string(s: &str) -> String {
    let s = s.trim();
    let mut prev = ' ';
    let mut result = String::new();

    for (i, c) in s.char_indices() {
        // Fix string for o. In the beginning, after punctuations etc it should be capital O
        if (c == 'o' || c == 'O') && (i == 0 || !prev.is_ascii_alphabetic()) {
            result.push('O');
        } else if c.is_ascii_alphabetic() {
            result.push(c.to_ascii_lowercase());
        }

        prev = c;
    }

    result
}

pub fn match_longest_common_prefix<'a>(trie: &Trie<u8>, prefix: &'a str) -> (&'a str, &'a str, bool) {
    if prefix.is_empty() {
        return ("", "", false);
    }

    let mut search = trie.inc_search();

    match search.query_until(prefix) {
        Ok(a) => {
            (&prefix, "", a.is_match())
        }
        Err(e) => {
            let matched = &prefix[0..e];
            let remaining = &prefix[e..];
            (matched, remaining, false)
        }
    }
}

pub struct MatchableNode<'a> {
    trie: &'a Trie<u8>,
    pos: LoudsNodeNum,
    complete: bool,
}

impl<'a> MatchableNode<'a> {
    pub fn from(trie: &'a Trie<u8>) -> Self {
        Self {
            trie,
            pos: LoudsNodeNum(1),
            complete: false,
        }
    }

    pub fn get_matching_node(&self, prefix: &str) -> Option<Self> {
        let mut m = IncSearch::resume(&self.trie.0, self.pos);

        match m.query_until(prefix) {
            Ok(a) => {
                if a.is_match() {
                    Some(Self {
                        trie: self.trie,
                        pos: Position::from(m),
                        complete: true,
                    })
                } else {
                    Some(MatchableNode { trie: self.trie, pos: Position::from(m), complete: false })
                }
            },
            Err(_) => {
                None
            }
        }
    }

    pub fn get_word(&self) -> Option<String> {
        if self.complete {
            let search = IncSearch::resume(&self.trie.0, self.pos);
            Some(search.prefix())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_string() {
        assert_eq!(fix_string("o"), "O");
        assert_eq!(fix_string("o!"), "O");
        assert_eq!(fix_string("o!o"), "OO");
        assert_eq!(fix_string("osomapto"), "Osomapto");
    }
}
