use crate::searcher::PatternSuggest;

pub struct Suggest {
    inner: PatternSuggest,
}

impl Suggest {
    pub fn new() -> Self {
        Suggest {
            inner: PatternSuggest::new(
                include_bytes!("patterns.fst"),
                include_bytes!("../../data/preprocessed-patterns.json"),
                include_str!("../../data/source-common-patterns.txt"),
            ),
        }
    }

    pub fn suggest(&self, input: &str) -> Vec<String> {
        self.inner.suggest(&fix_string(input))
    }
}

fn fix_string(s: &str) -> String {
    let s = s.trim();

    let mut result = String::new();
    let mut prev = ' '; // prev is non-alphabetic at first
    for c in s.chars() {
        // Fix string for o. In the beginning, after punctuations etc it should be capital O
        if (c == 'o' || c == 'O') && !prev.is_ascii_alphabetic() {
            result.push('O');
        } else if c.is_ascii_alphanumeric() || c == '`' {
            result.push(c.to_ascii_lowercase());
        }

        prev = c;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sort(mut vec: Vec<String>) -> Vec<String> {
        vec.sort();
        vec
    }

    #[test]
    fn test_fix_string() {
        assert_eq!(fix_string("o"), "O");
        assert_eq!(fix_string("o!"), "O");
        assert_eq!(fix_string("o!o"), "OO");
        assert_eq!(fix_string("osomapto"), "Osomapto");
        assert_eq!(fix_string("6t``"), "6t``");
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
