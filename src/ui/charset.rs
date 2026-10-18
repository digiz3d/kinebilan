#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Text,
    Date,
    /// Masked on screen. Copying its text is not allowed.
    Secret,
}

impl Charset {
    pub(super) fn accepts(self, c: char) -> bool {
        match self {
            Charset::Text | Charset::Secret => !c.is_control(),
            Charset::Date => c.is_ascii_digit() || c == '-',
        }
    }

    pub(super) fn max_len(self) -> usize {
        match self {
            Charset::Text | Charset::Secret => usize::MAX,
            Charset::Date => 10,
        }
    }
}
