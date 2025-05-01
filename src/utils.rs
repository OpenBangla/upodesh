fn is_convertible(c: char) -> bool {
    ((c >= 'a') && (c <= 'z')) || ((c >= 'A') && (c <= 'Z'))
}

pub fn fix_string(mut s: &str) -> String {
    let mut result = String::new();

    s = s.trim();
    let mut prev = None;

    for (i, c) in s.char_indices() {
        let mut make_lower = true;

        if !is_convertible(c) {
            prev = Some(c);
            continue;
        }

        // Fix string for o. In the beginning, after punctuations etc it should be capital O
        if c == 'o' || c == 'O' {
            if i == 0 {
                make_lower = false;
            } else if !is_convertible(prev.unwrap()) {
                make_lower = false;
            }
        }

        if make_lower {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c.to_ascii_uppercase());
        }

        prev = Some(c);
    }

    result
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
