#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Text,
    Date,
}

impl Charset {
    pub(super) fn accepts(self, c: char) -> bool {
        match self {
            Charset::Text => !c.is_control(),
            Charset::Date => c.is_ascii_digit() || c == '-',
        }
    }

    pub(super) fn max_len(self) -> usize {
        match self {
            Charset::Text => usize::MAX,
            Charset::Date => 10,
        }
    }
}
