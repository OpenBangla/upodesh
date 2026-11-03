use crate::khipro::classic::{Engine, instantiate_engine};
use std::cell::RefCell;

/// Compositional Khipro converter
pub struct Convert {
    engine: RefCell<Engine>,
}

impl Convert {
    pub fn new() -> Self {
        Self {
            engine: RefCell::new(instantiate_engine()),
        }
    }
    pub fn convert(&self, input: &str) -> String {
        self.engine.borrow_mut().convert(input)
    }
}

impl Default for Convert {
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
    fn test_convert_exact() {
        let c = Convert::new();
        let out = c.convert("bangla");
        assert_eq!(
            out.as_bytes(),
            b"\xe0\xa6\xac\xe0\xa6\xbe\xe0\xa6\x99\xe0\xa6\xb2\xe0\xa6\xbe"
        ); // বাংলা
    }
}
