use fst::{Set, SetBuilder};
use okkhor::parser::Parser;
use regex::Regex;


struct RegexSuggest {
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
        let rgx = Regex::new(&self.regex).unwrap();

        let r = self.words.search(rgx);

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading() {
        let suggest = RegexSuggest::new();
    }
}
