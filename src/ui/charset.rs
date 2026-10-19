#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Text,
    Date,
    /// Digits only, shown as `dd/mm/yyyy`.
    BirthDate,
    /// Masked on screen. Copying its text is not allowed.
    Secret,
}

impl Charset {
    pub(super) fn accepts(self, c: char) -> bool {
        match self {
            Charset::Text | Charset::Secret => !c.is_control(),
            Charset::Date => c.is_ascii_digit() || c == '-',
            Charset::BirthDate => c.is_ascii_digit(),
        }
    }

    /// How many more characters `value` can take.
    pub(super) fn room(self, value: &str) -> usize {
        match self {
            Charset::Text | Charset::Secret => usize::MAX,
            Charset::Date => 10_usize.saturating_sub(value.chars().count()),
            Charset::BirthDate => 8_usize.saturating_sub(value.chars().filter(char::is_ascii_digit).count()),
        }
    }
}

/// Rebuilds a birth date as `dd/mm/yyyy` from its digits. Returns the new text and where `cursor` lands in it,
/// which is right after the same number of digits as before.
pub(super) fn format_birth_date(value: &str, cursor: usize) -> (String, usize) {
    let digits_before = value[..cursor].chars().filter(char::is_ascii_digit).count().min(8);
    let mut out = String::new();
    let mut out_cursor = 0;
    for (i, digit) in value.chars().filter(char::is_ascii_digit).take(8).enumerate() {
        if i == 2 || i == 4 {
            out.push('/');
        }
        out.push(digit);
        if i + 1 == digits_before {
            out_cursor = out.len();
        }
    }
    (out, out_cursor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_birth_date_inserts_separators() {
        assert_eq!(format_birth_date("12", 2), ("12".into(), 2));
        assert_eq!(format_birth_date("123", 3), ("12/3".into(), 4));
        assert_eq!(format_birth_date("12041985", 8), ("12/04/1985".into(), 10));
    }

    #[test]
    fn test_birth_date_keeps_cursor_after_same_digits() {
        assert_eq!(format_birth_date("1/34", 1), ("13/4".into(), 1));
        assert_eq!(format_birth_date("125/34", 3), ("12/53/4".into(), 4));
    }

    #[test]
    fn test_birth_date_drops_extra_digits_and_symbols() {
        assert_eq!(format_birth_date("1204198512", 10), ("12/04/1985".into(), 10));
        assert_eq!(format_birth_date("", 0), (String::new(), 0));
    }
}
