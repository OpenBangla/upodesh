use crate::searcher::PatternSuggest;

pub struct Suggest {
    inner: PatternSuggest,
}

impl Suggest {
    pub fn new() -> Self {
        Suggest {
            inner: PatternSuggest::new(
                include_bytes!("patterns.fst"),
                include_bytes!("../../data/khipro-preprocessed-patterns.json"),
                include_str!("../../data/source-common-patterns.txt"),
            ),
        }
    }

    pub fn suggest(&self, input: &str) -> Vec<String> {
        self.inner.suggest(&fix_string(input))
    }
}

fn fix_string(s: &str) -> String {
    // Khipro is typed in lowercase only, the slicer (`/`), `,,` (nukta) and
    // `` ` `` (ZWNJ, ZWJ) are part of the words.
    s.trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | ',' | '`'))
        .map(|c| c.to_ascii_lowercase())
        .collect()
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
        assert_eq!(fix_string("  Ami  "), "ami");
        assert_eq!(fix_string("apn/i"), "apn/i");
        assert_eq!(fix_string("j,,!"), "j,,");
        assert_eq!(fix_string("6t``"), "6t``");
    }

    #[test]
    fn test_suggestions() {
        let suggest = Suggest::new();

        assert_eq!(sort(suggest.suggest("ami")), vec!["আমি"]);
        assert_eq!(sort(suggest.suggest("baxla")), vec!["বাংলা", "বাঙলা"]);
        assert_eq!(sort(suggest.suggest("sbadhinota")), vec!["স্বাধীনতা"]);
        assert_eq!(sort(suggest.suggest("duookh")), vec!["দুঃখ"]);
        assert_eq!(sort(suggest.suggest("cndrbindu")), vec!["চন্দ্রবিন্দু"]);
        assert_eq!(sort(suggest.suggest("bidzaloy")), vec!["বিদ্যালয়"]);
        assert_eq!(sort(suggest.suggest("bikkhwv")), vec!["বিক্ষোভ"]);
        assert_eq!(sort(suggest.suggest("valobasa")), vec!["ভালবাসা"]);
        assert_eq!(sort(suggest.suggest("ebx")), vec!["এবং"]);
        assert_eq!(sort(suggest.suggest("kotha")), vec!["কথা"]);
        assert_eq!(sort(suggest.suggest("cad")), vec!["চাঁদ", "ছাঁদ", "ছাদ"]);
        assert_eq!(
            sort(suggest.suggest("kormo")),
            vec!["করম", "কর্ম", "ক্রম", "কড়ম্ব"]
        );
        assert_eq!(sort(suggest.suggest("apn/i")), vec!["আপনি"]);
        assert_eq!(sort(suggest.suggest("ks/")), vec!["কষ", "কস", "কস্য"]);
    }

    #[test]
    fn test_fuzzy_suggestions() {
        let suggest = Suggest::new();

        // Modifier families (স/শ/ষ, র/ড়/ঢ়) and vowel length (ই/ঈ)
        assert_eq!(
            sort(suggest.suggest("sari")),
            vec!["শারি", "শারী", "শাড়ি", "শাড়ী", "সারি", "সারী", "সাড়ি", "সাড়ী"]
        );
        assert_eq!(sort(suggest.suggest("tik")), vec!["টিক", "ঠিক", "তিক"]);
        assert_eq!(sort(suggest.suggest("shesh")), vec!["শেষ", "সেস"]);
        assert_eq!(sort(suggest.suggest("shokti")), vec!["শকতি", "শক্তি", "সক্তি"]);
        // Similar sounding keys: জ/য, ং/ঙ
        assert_eq!(sort(suggest.suggest("jodi")), vec!["যদি"]);
        assert_eq!(sort(suggest.suggest("bangla")), vec!["বাংলা", "বাঙলা"]);
        // Consonants which may not be joined: missing `o` or slicer
        assert_eq!(sort(suggest.suggest("amra")), vec!["আমরা", "আমড়া"]);
        assert_eq!(sort(suggest.suggest("amora")), vec!["আমরা", "আমড়া"]);
        assert_eq!(sort(suggest.suggest("apni")), vec!["আপনি"]);
        assert_eq!(sort(suggest.suggest("ksh")), vec!["কশ", "কষ", "কস", "কস্য"]);
        // Common suffixes
        assert_eq!(sort(suggest.suggest("dukh")), vec!["দুঃখ", "দুখ"]);
    }

    #[test]
    fn test_explicit_modifiers_stay_exact() {
        let suggest = Suggest::new();

        assert_eq!(sort(suggest.suggest("tf")), vec!["ট"]);
        assert_eq!(sort(suggest.suggest("tfik")), vec!["টিক"]);
        assert_eq!(sort(suggest.suggest("kkhet")), vec!["ক্ষেত"]);
        assert_eq!(sort(suggest.suggest("kfet")), vec!["ক্ষেত"]);
        assert_eq!(sort(suggest.suggest("shesf")), vec!["শেষ"]);
    }

    #[test]
    fn test_empty_suggestion() {
        let suggest = Suggest::new();

        assert_eq!(suggest.suggest(""), Vec::<String>::new());
        assert_eq!(suggest.suggest("!!"), Vec::<String>::new());
        assert_eq!(suggest.suggest("6t``"), Vec::<String>::new());
        assert_eq!(suggest.suggest("t6th"), Vec::<String>::new());
    }
}
