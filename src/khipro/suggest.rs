use crate::bangla;
use crate::khipro::Convert;

/// Suggest on English Khipro input (converts then suggests on Bengali).
pub struct Suggest;

impl Suggest {
    pub fn new() -> Self {
        Self
    }
    pub fn suggest(&self, input: &str) -> Vec<String> {
        let composed = Convert::new().convert(input);
        bangla::suggest(&composed)
    }
}

impl Default for Suggest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Exact byte equality: Bengali combining marks (U+09BE aa-kar) normalize
    // differently from literal source text, so compare engine snapshot bytes.
    #[test]
    fn test_suggest_exact() {
        let s = Suggest::new();
        let out = s.suggest("ami");
        assert!(!out.is_empty());
        let first = out.first().expect("non-empty");
        dbg!(first);
        assert_eq!(
            first.as_bytes(),
            b"\xe0\xa6\x86\xe0\xa6\xae\xe0\xa6\xbf\xe0\xa6\xb0"
        ); // আমিন
    }
}
