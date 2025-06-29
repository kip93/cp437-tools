//! CP437 to/from UTF-8.

use indexmap::IndexMap;
use lazy_static::lazy_static;

lazy_static! {
    /// An array of 256 elements, mapping most of the CP437 values to UTF-8 characters.
    ///
    /// Mostly follows CP437, except for:
    ///  * 0x0A & 0x0D are kept for use as line endings.
    ///  * 0x1A is used for SAUCE.
    ///  * 0x1B is used for ANSI escape sequences.
    ///
    /// These exclusions should be fine since most programs can't even use them
    /// without issues. And this makes rendering simpler too.
    ///
    /// See <https://en.wikipedia.org/wiki/Code_page_437#Character_set>
    ///
    #[rustfmt::skip]
    pub static ref CP437_TO_UTF8: &'static [char] = &[
        /* XX    X0   X1   X2   X3   X4   X5   X6   X7    X8   X9   XA    XB   XC    XD    XE   XF */
        /* 0X */ '\0', '☺', '☻', '♥', '♦', '♣', '♠', '•',  '◘', '○', '\n', '♂', '♀',  '\r', '♫', '☼',
        /* 1X */ '►',  '◄', '↕', '‼', '¶', '§', '▬', '↨',  '↑', '↓', '',  '', '∟',  '↔',  '▲', '▼',
        /* 2X */ ' ',  '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*',  '+', ',',  '-',  '.', '/',
        /* 3X */ '0',  '1', '2', '3', '4', '5', '6', '7',  '8', '9', ':',  ';', '<',  '=',  '>', '?',
        /* 4X */ '@',  'A', 'B', 'C', 'D', 'E', 'F', 'G',  'H', 'I', 'J',  'K', 'L',  'M',  'N', 'O',
        /* 5X */ 'P',  'Q', 'R', 'S', 'T', 'U', 'V', 'W',  'X', 'Y', 'Z',  '[', '\\', ']',  '^', '_',
        /* 6X */ '`',  'a', 'b', 'c', 'd', 'e', 'f', 'g',  'h', 'i', 'j',  'k', 'l',  'm',  'n', 'o',
        /* 7X */ 'p',  'q', 'r', 's', 't', 'u', 'v', 'w',  'x', 'y', 'z',  '{', '|',  '}',  '~', '⌂',
        /* 8X */ 'Ç',  'ü', 'é', 'â', 'ä', 'à', 'å', 'ç',  'ê', 'ë', 'è',  'ï', 'î',  'ì',  'Ä', 'Å',
        /* 9X */ 'É',  'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù',  'ÿ', 'Ö', 'Ü',  '¢', '£',  '¥',  '₧', 'ƒ',
        /* AX */ 'á',  'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º',  '¿', '⌐', '¬',  '½', '¼',  '¡',  '«', '»',
        /* BX */ '░',  '▒', '▓', '│', '┤', '╡', '╢', '╖',  '╕', '╣', '║',  '╗', '╝',  '╜',  '╛', '┐',
        /* CX */ '└',  '┴', '┬', '├', '─', '┼', '╞', '╟',  '╚', '╔', '╩',  '╦', '╠',  '═',  '╬', '╧',
        /* DX */ '╨',  '╤', '╥', '╙', '╘', '╒', '╓', '╫',  '╪', '┘', '┌',  '█', '▄',  '▌',  '▐', '▀',
        /* EX */ 'α',  'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ',  'Φ', 'Θ', 'Ω',  'δ', '∞',  'φ',  'ε', '∩',
        /* FX */ '≡',  '±', '≥', '≤', '⌠', '⌡', '÷', '≈',  '°', '∙', '·',  '√', 'ⁿ',  '²',  '■', ' ',
    ];

    /// A dictionary of 256 elements, mapping selected UTF-8 characters to corresponding CP437.
    ///
    /// Effectively the inverse of [`CP437_TO_UTF8`].
    ///
    pub static ref UTF8_TO_CP437: IndexMap<char, u8> =
        CP437_TO_UTF8
            .iter()
            .enumerate()
            .map(|(a, b)| return (*b, u8::try_from(a).expect("Spec only has 256 values"))).collect::<IndexMap<_, _>>();
}

/// Apply [`struct@CP437_TO_UTF8`] to the given bytes.
#[must_use]
pub fn to_utf8(cp437: &[u8]) -> String {
    return cp437.iter().map(|byte| return CP437_TO_UTF8[*byte as usize]).collect();
}

/// Apply [`struct@UTF8_TO_CP437`] to the given string.
///
/// # Errors
///
/// Fails when there's no equivalent UTF-8 -> CP437 character.
///
pub fn to_cp437(utf8: &str) -> Result<Vec<u8>, String> {
    return utf8
        .chars()
        .map(|r#char| {
            return UTF8_TO_CP437.get(&r#char).map(|byte| return *byte).ok_or_else(|| {
                return format!("{} (U+{:X}) is not a valid CP437 character", r#char, r#char as u32);
            });
        })
        .collect::<Result<Vec<u8>, String>>();
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn cp437_to_utf8() {
        for i in 0x00..=0xFF {
            assert_eq!(UTF8_TO_CP437.get(&CP437_TO_UTF8[i]), Some(i as u8).as_ref());
        }
    }

    #[test]
    fn utf8_to_cp437() {
        for c in &CP437_TO_UTF8[0x00..=0xFF] {
            assert_eq!(CP437_TO_UTF8[*UTF8_TO_CP437.get(c).unwrap() as usize], *c);
        }
    }

    #[test]
    fn vec_to_utf8() {
        assert_eq!(to_utf8(&[0x01]), "☺");
    }

    #[test]
    fn str_to_cp437_ok() {
        let result = to_cp437("☺");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0x01]);
    }

    #[test]
    fn str_to_cp437_err() {
        let result = to_cp437("🚫");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "🚫 (U+1F6AB) is not a valid CP437 character");
    }
}
