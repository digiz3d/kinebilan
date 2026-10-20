#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Text,
    /// Digits only, shown as `dd/mm/yyyy`.
    Date,
    /// Digits only, shown as `dd/mm/yyyy HH:MM`.
    DateTime,
    /// Masked on screen. Copying its text is not allowed.
    Secret,
}

impl Charset {
    /// The shape of a digits-only charset. Letters are digit slots, anything else is a separator.
    pub(super) fn template(self) -> Option<&'static str> {
        match self {
            Charset::Text | Charset::Secret => None,
            Charset::Date => Some("dd/mm/yyyy"),
            Charset::DateTime => Some("dd/mm/yyyy HH:MM"),
        }
    }

    pub(super) fn accepts(self, c: char) -> bool {
        match self {
            Charset::Text | Charset::Secret => !c.is_control(),
            Charset::Date | Charset::DateTime => c.is_ascii_digit(),
        }
    }

    /// How many more characters `value` can take.
    pub(super) fn room(self, value: &str) -> usize {
        match self.template() {
            None => usize::MAX,
            Some(template) => digit_slots(template).saturating_sub(value.chars().filter(char::is_ascii_digit).count()),
        }
    }
}

fn digit_slots(template: &str) -> usize {
    template.chars().filter(char::is_ascii_alphabetic).count()
}

/// Rebuilds `value` in the shape of `template` from its digits. Returns the new text and where `cursor` lands in it,
/// which is right after the same number of digits as before.
pub(super) fn format_digits(template: &str, value: &str, cursor: usize) -> (String, usize) {
    let slots = digit_slots(template);
    let mut separators = Vec::new();
    let mut seen = 0;
    for c in template.chars() {
        if c.is_ascii_alphabetic() {
            seen += 1;
        } else {
            separators.push((seen, c));
        }
    }
    let digits_before = value[..cursor].chars().filter(char::is_ascii_digit).count().min(slots);
    let mut out = String::new();
    let mut out_cursor = 0;
    for (i, digit) in value.chars().filter(char::is_ascii_digit).take(slots).enumerate() {
        for &(at, separator) in &separators {
            if at == i {
                out.push(separator);
            }
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
    fn test_date_inserts_separators() {
        assert_eq!(format_digits("dd/mm/yyyy", "12", 2), ("12".into(), 2));
        assert_eq!(format_digits("dd/mm/yyyy", "123", 3), ("12/3".into(), 4));
        assert_eq!(format_digits("dd/mm/yyyy", "12041985", 8), ("12/04/1985".into(), 10));
    }

    #[test]
    fn test_datetime_inserts_separators() {
        assert_eq!(format_digits("dd/mm/yyyy HH:MM", "201020261615", 12), ("20/10/2026 16:15".into(), 16));
        assert_eq!(format_digits("dd/mm/yyyy HH:MM", "20102026", 8), ("20/10/2026".into(), 10));
    }

    #[test]
    fn test_date_keeps_cursor_after_same_digits() {
        assert_eq!(format_digits("dd/mm/yyyy", "1/34", 1), ("13/4".into(), 1));
        assert_eq!(format_digits("dd/mm/yyyy", "125/34", 3), ("12/53/4".into(), 4));
    }

    #[test]
    fn test_date_drops_extra_digits_and_symbols() {
        assert_eq!(format_digits("dd/mm/yyyy", "1204198512", 10), ("12/04/1985".into(), 10));
        assert_eq!(format_digits("dd/mm/yyyy", "", 0), (String::new(), 0));
    }
}
