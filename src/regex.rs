use fst::{IntoStreamer, Set};
use okkhor::parser::Parser;

use crate::automation::DfaFstAutomaton;


pub struct RegexSuggest {
    parser: Parser,
    regex: String,
    words: Set<Vec<u8>>,
}

impl RegexSuggest {
    pub fn new() -> Self {
        let list = include_str!("../data/source-words.txt");

        let mut list = list.lines().map(|s| s.trim()).collect::<Vec<_>>();

        list.sort();

        let words = Set::from_iter(list).unwrap();
        
        Self {
            parser: Parser::new_regex(),
            regex: String::with_capacity(1024),
            words,
        }
    }

    pub fn suggest(&mut self, input: &str) -> Vec<String> {
        self.parser.convert_regex_into(input, &mut self.regex);
        let rgx = DfaFstAutomaton::new(&self.regex);

        let words = self.words.search(rgx).into_stream().into_strs().unwrap();

        words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading() {
        let mut suggest = RegexSuggest::new();

        assert_eq!(suggest.suggest("shari"), ["শ\u{9be}রি", "শ\u{9be}রী", "শ\u{9be}ড়ি", "শ\u{9be}ড়ী", "স\u{9be}রি", "স\u{9be}রী", "স\u{9be}ড়ি", "স\u{9be}ড়ী"]);
    }
}
