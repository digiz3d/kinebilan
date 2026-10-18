use sdl3::ttf::Font;

use super::theme::{PAD_X, PAD_Y};

pub fn line_height(font: &Font) -> i32 {
    font.recommended_line_spacing()
}

pub fn measure(font: &Font, s: &str) -> (i32, i32) {
    font.size_of(s)
        .map(|(w, h)| (w as i32, h as i32))
        .unwrap_or((0, line_height(font)))
}

pub(super) fn wrap_spans(font: &Font, text: &str, max_w: i32) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut base = 0;
    for paragraph in text.split('\n') {
        let mut line = None;
        let mut start = base;
        for word in paragraph.split(' ') {
            let end = start + word.len();
            line = Some(match line {
                Some((s, _)) if measure(font, &text[s..end]).0 <= max_w => (s, end),
                Some(done) => {
                    lines.push(done);
                    (start, end)
                }
                None => (start, end),
            });
            start = end + 1;
        }
        lines.extend(line);
        base += paragraph.len() + 1;
    }
    lines
}

pub fn wrap(font: &Font, text: &str, max_w: i32) -> Vec<String> {
    wrap_spans(font, text, max_w)
        .into_iter()
        .map(|(s, e)| text[s..e].to_string())
        .collect()
}

pub(super) fn line_of(lines: &[(usize, usize)], cursor: usize) -> usize {
    lines.iter().position(|&(_, end)| cursor <= end).unwrap_or(lines.len() - 1)
}

pub(super) fn word_span(value: &str, i: usize) -> (usize, usize) {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let start = value[..i]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| is_word(c))
        .last()
        .map_or(i, |(j, _)| j);
    let end = value[i..]
        .char_indices()
        .take_while(|&(_, c)| is_word(c))
        .last()
        .map_or(i, |(j, c)| i + j + c.len_utf8());
    (start, end)
}

pub(super) fn index_at(font: &Font, value: &str, (start, end): (usize, usize), x: i32) -> usize {
    let text = &value[start..end];
    text.char_indices()
        .map(|(i, _)| i)
        .chain([text.len()])
        .min_by_key(|&i| (measure(font, &text[..i]).0 - x).abs())
        .map_or(start, |i| start + i)
}

pub fn area_height(font: &Font, value: &str, w: i32, min_rows: usize) -> i32 {
    let rows = if value.is_empty() {
        1
    } else {
        wrap_spans(font, value, w - 2 * PAD_X).len()
    };
    rows.max(min_rows) as i32 * line_height(font) + 2 * PAD_Y
}

#[cfg(test)]
mod tests {
    use super::word_span;

    #[test]
    fn test_word_span_selects_surrounding_word() {
        assert_eq!(word_span("Jean Dupont", 6), (5, 11));
        assert_eq!(word_span("Jean Dupont", 1), (0, 4));
    }

    #[test]
    fn test_word_span_at_boundary_selects_left_word() {
        assert_eq!(word_span("Jean Dupont", 4), (0, 4));
    }

    #[test]
    fn test_word_span_handles_accents() {
        assert_eq!(word_span("Prénom test", 4), (0, 7));
    }
}
