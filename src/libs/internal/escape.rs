use regex::{Captures, Regex};

pub fn escape(value: String) -> String {
    let value =
        Regex::new(r"\\x([0-9A-Fa-f]{1,2})")
            .unwrap()
            .replace_all(&value, |groups: &Captures| {
                return (u8::from_str_radix(&groups[1], 16).unwrap() as char).to_string();
            });
    let value =
        Regex::new(r"\\u([0-9A-Fa-f]{1,6})")
            .unwrap()
            .replace_all(&value, |groups: &Captures| {
                return char::from_u32(u32::from_str_radix(&groups[1], 16).unwrap())
                    .unwrap()
                    .to_string();
            });
    let value = value
        .replace("\\0", "\0")
        .replace("\\t", "\t")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\\\", "\\");

    return value;
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn empty() {
        check("", "");
    }

    #[test]
    fn no_escape() {
        check("X", "X");
    }

    #[test]
    fn ascii() {
        check("\\x40", "@");
    }

    #[test]
    fn unicode() {
        check("\\u263A", "☺");
    }

    #[test]
    fn others() {
        check("\\0\\t\\n\\r\\\\", "\0\t\n\r\\");
    }

    fn check(input: &str, output: &str) {
        assert_eq!(escape(String::from(input)), output);
    }
}
