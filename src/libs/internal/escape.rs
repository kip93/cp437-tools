use regex::{Captures, Regex};

pub fn escape(haystack: &str) -> Result<String, String> {
    let unescape = |groups: &Captures| -> Result<String, String> {
        return Ok(String::from(if groups.get(1).is_some() {
            '\0'
        } else if groups.get(2).is_some() {
            '\t'
        } else if groups.get(3).is_some() {
            '\n'
        } else if groups.get(4).is_some() {
            '\r'
        } else if let Some(x) = groups.get(5) {
            u8::from_str_radix(x.as_str(), 16).expect("Already checked by regex") as char
        } else if let Some(x) = groups.get(6) {
            char::from_u32(u32::from_str_radix(x.as_str(), 16).expect("Already checked by regex"))
                .ok_or(format!("Invalid unicode: {}", x.as_str()))?
        } else {
            '\\'
        }));
    };

    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for groups in Regex::new(r"(\\0)|(\\t)|(\\n)|(\\r)|\\x([0-9A-Fa-f]{1,2})|\\u([0-9A-Fa-f]{1,6})|(\\\\)")
        .expect("Valid regex")
        .captures_iter(haystack)
    {
        let r#match = groups.get(0).expect("Guaranteed by library");
        new.push_str(&haystack[last_match..r#match.start()]);
        new.push_str(&unescape(&groups)?);
        last_match = r#match.end();
    }
    new.push_str(&haystack[last_match..]);

    return Ok(new);
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn empty() -> Result<(), String> {
        return check("", "");
    }

    #[test]
    fn no_escape() -> Result<(), String> {
        return check("X", "X");
    }

    #[test]
    fn ascii() -> Result<(), String> {
        return check("\\x40", "@");
    }

    #[test]
    fn unicode() -> Result<(), String> {
        return check("\\u263A", "☺");
    }

    #[test]
    fn double() -> Result<(), String> {
        return check("\\\\0", "\\0");
    }

    #[test]
    fn triple() -> Result<(), String> {
        return check("\\\\\\0", "\\\0");
    }

    #[test]
    fn others() -> Result<(), String> {
        return check("\\0\\t\\n\\r\\\\", "\0\t\n\r\\");
    }

    #[test]
    fn invalid() {
        assert!(escape("\\u110000").is_err());
    }

    fn check(input: &str, output: &str) -> Result<(), String> {
        return escape(input).map(|input| assert_eq!(input, output));
    }
}
