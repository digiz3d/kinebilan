use crate::focus::Field;

#[derive(Default)]
pub struct Caret {
    owner: Option<Field>,
    anchor: usize,
    pub(super) cursor: usize,
    pub(super) scroll: i32,
}

impl Caret {
    pub(super) fn sel(&self) -> (usize, usize) {
        (self.anchor.min(self.cursor), self.anchor.max(self.cursor))
    }

    pub(super) fn claim(&mut self, field: Field, value: &str) {
        if self.owner != Some(field) {
            *self = Caret { owner: Some(field), anchor: value.len(), cursor: value.len(), scroll: 0 };
        }
    }

    pub(super) fn fit(&mut self, value: &str) {
        self.anchor = floor_boundary(value, self.anchor);
        self.cursor = floor_boundary(value, self.cursor);
    }

    pub(super) fn select_to(&mut self, i: usize, extend: bool) {
        self.cursor = i;
        if !extend {
            self.anchor = i;
        }
    }

    pub(super) fn nav(&mut self, value: &str, forward: bool, extend: bool) {
        let (lo, hi) = self.sel();
        let to = if !extend && lo != hi {
            if forward { hi } else { lo }
        } else if forward {
            self.cursor + value[self.cursor..].chars().next().map_or(0, char::len_utf8)
        } else {
            self.cursor - value[..self.cursor].chars().next_back().map_or(0, char::len_utf8)
        };
        self.select_to(to, extend);
    }

    pub(super) fn delete_sel(&mut self, value: &mut String) -> bool {
        let (lo, hi) = self.sel();
        if lo == hi {
            return false;
        }
        value.replace_range(lo..hi, "");
        self.select_to(lo, false);
        true
    }

    pub(super) fn delete(&mut self, value: &mut String, back: bool) -> bool {
        self.delete_sel(value) || {
            self.nav(value, !back, true);
            self.delete_sel(value)
        }
    }

    pub(super) fn insert(&mut self, value: &mut String, text: &str) {
        value.insert_str(self.cursor, text);
        self.select_to(self.cursor + text.len(), false);
    }
}

fn floor_boundary(value: &str, mut i: usize) -> usize {
    i = i.min(value.len());
    while !value.is_char_boundary(i) {
        i -= 1;
    }
    i
}
