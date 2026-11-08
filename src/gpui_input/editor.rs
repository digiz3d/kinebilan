//! Text editing over a shared `Entity<String>`.

use std::ops::Range;
use std::time::Duration;

use gpui::{
    App, Bounds, Context, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Focusable,
    InteractiveElement, LayoutId, PaintQuad, Pixels, ShapedLine, SharedString, Subscription, Task,
    TextRun, UTF16Selection, Window, fill, hsla, point, prelude::*, px, relative, size,
};
use unicode_segmentation::*;

use super::{
    Backspace, Copy, Cut, Delete, Down, End, Home, Left, Newline, Paste, Redo, Right, SelectAll,
    SelectLeft, SelectRight, Undo, Up, WordLeft, WordRight,
};

pub struct Editor {
    pub value: Entity<String>,
    pub placeholder: SharedString,
    pub secret: bool,
    pub multiline: bool,
    mask: Option<&'static str>,
    wrap_width: Pixels,
    scroll_x: Pixels,
    pub focus_handle: FocusHandle,
    pub cursor: usize,
    pub anchor: usize,
    undo: Vec<(String, usize, usize)>,
    redo: Vec<(String, usize, usize)>,
    bounds: Option<Bounds<Pixels>>,
    pub cursor_visible: bool,
    _blink_task: Task<()>,
    _subscriptions: Vec<Subscription>,
}

impl Editor {
    pub fn over(
        value: Entity<String>,
        placeholder: SharedString,
        secret: bool,
        multiline: bool,
        mask: Option<&'static str>,
        wrap_width: Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();

        let focus_sub = cx.on_focus(&focus_handle, window, |this, _window, cx| {
            this.start_blink(cx);
        });
        let blur_sub = cx.on_blur(&focus_handle, window, |this, _window, cx| {
            this.stop_blink(cx);
        });

        // External writes can invalidate cursor and selection boundaries.
        let value_sub = cx.observe(&value, |this, value, cx| {
            let content = value.read(cx);
            let mut cursor = this.cursor.min(content.len());
            while cursor > 0 && !content.is_char_boundary(cursor) {
                cursor -= 1;
            }
            this.cursor = floor_grapheme_boundary(content, cursor);
            this.anchor = floor_grapheme_boundary(content, this.anchor.min(content.len()));
        });
        let cursor = value.read(cx).len();

        Self {
            value,
            placeholder,
            secret,
            multiline,
            mask,
            wrap_width,
            scroll_x: px(0.),
            focus_handle,
            cursor,
            anchor: cursor,
            undo: Vec::new(),
            redo: Vec::new(),
            bounds: None,
            cursor_visible: false,
            _blink_task: Task::ready(()),
            _subscriptions: vec![focus_sub, blur_sub, value_sub],
        }
    }

    pub fn text(&self, cx: &App) -> String {
        self.value.read(cx).clone()
    }

    fn start_blink(&mut self, cx: &mut Context<Self>) {
        self.cursor_visible = true;
        self._blink_task = Self::spawn_blink_task(cx);
    }

    fn stop_blink(&mut self, cx: &mut Context<Self>) {
        self.cursor_visible = false;
        self._blink_task = Task::ready(());
        cx.notify();
    }

    fn spawn_blink_task(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                let result = this.update(cx, |editor, cx| {
                    editor.cursor_visible = !editor.cursor_visible;
                    cx.notify();
                });
                if result.is_err() {
                    break;
                }
            }
        })
    }

    fn reset_blink(&mut self, cx: &mut Context<Self>) {
        self.cursor_visible = true;
        self._blink_task = Self::spawn_blink_task(cx);
    }

    fn record_undo(&mut self, value: String) {
        self.undo.push((value, self.anchor, self.cursor));
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn restore(&mut self, snapshot: (String, usize, usize), cx: &mut Context<Self>) {
        let current = self.text(cx);
        self.redo.push((current, self.anchor, self.cursor));
        let (text, anchor, cursor) = snapshot;
        self.anchor = anchor.min(text.len());
        self.cursor = cursor.min(text.len());
        self.value.update(cx, |value, cx| {
            *value = text;
            cx.notify();
        });
        cx.notify();
    }

    pub fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(snapshot) = self.undo.pop() {
            self.restore(snapshot, cx);
        }
    }

    pub fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(snapshot) = self.redo.pop() {
            let current = self.text(cx);
            self.undo.push((current, self.anchor, self.cursor));
            let (text, anchor, cursor) = snapshot;
            self.anchor = anchor.min(text.len());
            self.cursor = cursor.min(text.len());
            self.value.update(cx, |value, cx| {
                *value = text;
                cx.notify();
            });
            cx.notify();
        }
    }

    fn move_cursor(&mut self, forward: bool, extend: bool, cx: &mut Context<Self>) {
        let content = self.text(cx);
        let (lo, hi) = (self.anchor.min(self.cursor), self.anchor.max(self.cursor));
        self.cursor = if !extend && lo != hi {
            if forward { hi } else { lo }
        } else if forward {
            next_boundary(&content, self.cursor)
        } else {
            previous_boundary(&content, self.cursor)
        };
        if !extend {
            self.anchor = self.cursor;
        }
        self.reset_blink(cx);
        cx.notify();
    }

    pub fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(false, false, cx);
    }
    pub fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(false, true, cx);
    }

    pub fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(true, false, cx);
    }
    pub fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(true, true, cx);
    }

    fn move_word(&mut self, forward: bool, cx: &mut Context<Self>) {
        let content = self.text(cx);
        let is_word = |c: char| c.is_alphanumeric() || c == '_';
        self.cursor = if forward {
            let rest = &content[self.cursor..];
            let start = rest.find(is_word).unwrap_or(rest.len());
            let word = &rest[start..];
            self.cursor + start + word.find(|c: char| !is_word(c)).unwrap_or(word.len())
        } else {
            content[..self.cursor]
                .trim_end_matches(|c: char| !is_word(c))
                .trim_end_matches(is_word)
                .len()
        };
        self.anchor = self.cursor;
        self.reset_blink(cx);
        cx.notify();
    }

    pub fn word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_word(false, cx);
    }
    pub fn word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_word(true, cx);
    }

    fn move_vertical(&mut self, down: bool, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.text(cx);
        let display = if self.secret {
            "•".repeat(content.chars().count())
        } else {
            content.clone()
        };
        let lines =
            shape_display_lines(&display, self.multiline.then_some(self.wrap_width), window);
        let offset = if self.secret {
            content[..self.cursor].chars().count() * "•".len()
        } else {
            self.cursor
        };
        let row = line_index_for_offset(&lines, &display, offset);
        let Some((line, start)) = lines.get(row) else {
            return;
        };
        let x = line.x_for_index(offset.saturating_sub(*start));
        let target_row = if down { row + 1 } else { row.saturating_sub(1) };
        if target_row == row {
            return;
        }
        if let Some((target, target_start)) = lines.get(target_row) {
            let index = *target_start + target.closest_index_for_x(x);
            self.cursor = if self.secret {
                let chars = display[..index].chars().count();
                content
                    .char_indices()
                    .nth(chars)
                    .map_or(content.len(), |(byte, _)| byte)
            } else {
                index
            };
            self.anchor = self.cursor;
            self.reset_blink(cx);
            cx.notify();
        }
    }

    pub fn up(&mut self, _: &Up, window: &mut Window, cx: &mut Context<Self>) {
        self.move_vertical(false, window, cx);
    }
    pub fn down(&mut self, _: &Down, window: &mut Window, cx: &mut Context<Self>) {
        self.move_vertical(true, window, cx);
    }

    pub fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.anchor = 0;
        self.cursor = self.text(cx).len();
        cx.notify();
    }

    pub fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if self.secret {
            return;
        }
        let (lo, hi) = (self.anchor.min(self.cursor), self.anchor.max(self.cursor));
        if lo < hi {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.text(cx)[lo..hi].to_string(),
            ));
        }
    }

    pub fn cut(&mut self, _action: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.secret {
            self.copy(&Copy, window, cx);
        }
        self.backspace(&Backspace, window, cx);
    }

    pub fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let mut text = text.replace("\r\n", "\n");
        if !self.multiline {
            text.retain(|ch| ch != '\n');
        }
        if self.mask.is_some() {
            text.retain(|ch| ch.is_ascii_digit());
        }
        let (lo, hi) = (self.anchor.min(self.cursor), self.anchor.max(self.cursor));
        let inserted = text;
        if inserted.is_empty() {
            return;
        }
        self.record_undo(self.text(cx));
        let raw = self.text(cx);
        let raw = raw[..lo].to_owned() + &inserted + &raw[hi..];
        let (value, cursor) = match self.mask {
            Some(mask) => format_digits(mask, &raw, lo + inserted.len()),
            None => (raw, lo + inserted.len()),
        };
        self.value.update(cx, |s, cx| {
            *s = value;
            cx.notify();
        });
        self.cursor = cursor;
        self.anchor = self.cursor;
        self.reset_blink(cx);
        cx.notify();
    }

    pub fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.cursor = 0;
        self.anchor = 0;
        self.reset_blink(cx);
        cx.notify();
    }

    pub fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.cursor = self.text(cx).len();
        self.anchor = self.cursor;
        self.reset_blink(cx);
        cx.notify();
    }

    pub fn newline(&mut self, _: &Newline, _: &mut Window, cx: &mut Context<Self>) {
        self.record_undo(self.text(cx));
        let cursor = self.cursor;
        let start = self.anchor.min(cursor);
        let end = self.anchor.max(cursor);
        self.value.update(cx, |s, cx| {
            s.replace_range(start..end, "\n");
            cx.notify();
        });
        self.cursor = start + 1;
        self.anchor = self.cursor;
        self.reset_blink(cx);
        cx.notify();
    }

    pub fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        let content = self.text(cx);
        if self.anchor == self.cursor
            && self.mask.is_some()
            && content[..self.cursor]
                .chars()
                .next_back()
                .is_some_and(|ch| !ch.is_ascii_digit())
        {
            self.cursor = previous_boundary(&content, self.cursor);
            self.anchor = self.cursor;
        }
        let (lo, hi) = (self.anchor.min(self.cursor), self.anchor.max(self.cursor));
        if lo != hi || self.cursor > 0 {
            self.record_undo(content.clone());
            let start = if lo != hi {
                lo
            } else {
                previous_boundary(&content, self.cursor)
            };
            let end = if lo != hi { hi } else { self.cursor };
            let mut updated = content;
            updated.replace_range(start..end, "");
            let (updated, cursor) = match self.mask {
                Some(mask) => format_digits(mask, &updated, start),
                None => (updated, start),
            };
            self.value.update(cx, |s, cx| {
                *s = updated;
                cx.notify();
            });
            self.cursor = cursor;
            self.anchor = cursor;
        }
        self.reset_blink(cx);
        cx.notify();
    }

    pub fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        let content = self.text(cx);
        if self.anchor == self.cursor
            && self.mask.is_some()
            && content[self.cursor..]
                .chars()
                .next()
                .is_some_and(|ch| !ch.is_ascii_digit())
        {
            self.cursor = next_boundary(&content, self.cursor);
            self.anchor = self.cursor;
            cx.notify();
            return;
        }
        let (lo, hi) = (self.anchor.min(self.cursor), self.anchor.max(self.cursor));
        if lo != hi || self.cursor < content.len() {
            self.record_undo(content.clone());
            let start = if lo != hi { lo } else { self.cursor };
            let end = if lo != hi {
                hi
            } else {
                next_boundary(&content, self.cursor)
            };
            let mut updated = content;
            updated.replace_range(start..end, "");
            let (updated, cursor) = match self.mask {
                Some(mask) => format_digits(mask, &updated, start),
                None => (updated, start),
            };
            self.value.update(cx, |s, cx| {
                *s = updated;
                cx.notify();
            });
            self.cursor = cursor;
            self.anchor = cursor;
        }
        self.reset_blink(cx);
        cx.notify();
    }
}

fn previous_boundary(content: &str, offset: usize) -> usize {
    content
        .grapheme_indices(true)
        .rev()
        .find_map(|(idx, _)| (idx < offset).then_some(idx))
        .unwrap_or(0)
}

fn floor_grapheme_boundary(content: &str, offset: usize) -> usize {
    if offset >= content.len() {
        return content.len();
    }
    content
        .grapheme_indices(true)
        .map(|(index, _)| index)
        .take_while(|&index| index <= offset)
        .last()
        .unwrap_or(0)
}

fn ceil_grapheme_boundary(content: &str, offset: usize) -> usize {
    let floor = floor_grapheme_boundary(content, offset);
    if floor == offset {
        offset
    } else {
        next_boundary(content, floor)
    }
}

fn next_boundary(content: &str, offset: usize) -> usize {
    content
        .grapheme_indices(true)
        .find_map(|(idx, _)| (idx > offset).then_some(idx))
        .unwrap_or(content.len())
}

fn format_digits(template: &str, value: &str, cursor: usize) -> (String, usize) {
    let slots = template.chars().filter(char::is_ascii_alphabetic).count();
    let mut separators = Vec::new();
    let mut seen = 0;
    for ch in template.chars() {
        if ch.is_ascii_alphabetic() {
            seen += 1;
        } else {
            separators.push((seen, ch));
        }
    }
    let digits_before = value[..cursor]
        .chars()
        .filter(char::is_ascii_digit)
        .count()
        .min(slots);
    let mut result = String::new();
    let mut result_cursor = 0;
    for (index, digit) in value
        .chars()
        .filter(char::is_ascii_digit)
        .take(slots)
        .enumerate()
    {
        for &(at, separator) in &separators {
            if at == index {
                result.push(separator);
            }
        }
        result.push(digit);
        if index + 1 == digits_before {
            result_cursor = result.len();
        }
    }
    (result, result_cursor)
}

fn offset_from_utf16(content: &str, offset: usize) -> usize {
    let mut utf8_offset = 0;
    let mut utf16_count = 0;
    for ch in content.chars() {
        if utf16_count >= offset {
            break;
        }
        utf16_count += ch.len_utf16();
        utf8_offset += ch.len_utf8();
    }
    utf8_offset
}

fn offset_to_utf16(content: &str, offset: usize) -> usize {
    let mut utf16_offset = 0;
    let mut utf8_count = 0;
    for ch in content.chars() {
        if utf8_count >= offset {
            break;
        }
        utf8_count += ch.len_utf8();
        utf16_offset += ch.len_utf16();
    }
    utf16_offset
}

fn range_to_utf16(content: &str, range: &Range<usize>) -> Range<usize> {
    offset_to_utf16(content, range.start)..offset_to_utf16(content, range.end)
}

fn range_from_utf16(content: &str, range_utf16: &Range<usize>) -> Range<usize> {
    offset_from_utf16(content, range_utf16.start)..offset_from_utf16(content, range_utf16.end)
}

impl Focusable for Editor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for Editor {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        let content = self.text(cx);
        let range = range_from_utf16(&content, &range_utf16);
        actual_range.replace(range_to_utf16(&content, &range));
        Some(content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let content = self.text(cx);
        let (start, end) = (self.anchor.min(self.cursor), self.anchor.max(self.cursor));
        Some(UTF16Selection {
            range: offset_to_utf16(&content, start)..offset_to_utf16(&content, end),
            reversed: self.cursor < self.anchor,
        })
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let content = self.text(cx);
        let start = offset_from_utf16(&content, range_utf16.start);
        let end = offset_from_utf16(&content, range_utf16.end);
        self.anchor = floor_grapheme_boundary(&content, start);
        self.cursor = if start == end {
            self.anchor
        } else {
            ceil_grapheme_boundary(&content, end)
        };
        cx.notify();
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let content = self.text(cx);
        let range = range_utf16
            .as_ref()
            .map(|r| {
                let range = range_from_utf16(&content, r);
                floor_grapheme_boundary(&content, range.start)
                    ..ceil_grapheme_boundary(&content, range.end)
            })
            .unwrap_or(self.cursor..self.cursor);

        let replacement = if self.mask.is_some() {
            new_text
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
        } else if self.multiline {
            new_text.to_string()
        } else {
            new_text.replace('\n', "")
        };
        let raw_content = content[..range.start].to_owned() + &replacement + &content[range.end..];
        let raw_cursor = range.start + replacement.len();
        let (new_content, cursor) = match self.mask {
            Some(mask) => format_digits(mask, &raw_content, raw_cursor),
            None => (raw_content, raw_cursor),
        };
        if new_content != content {
            self.record_undo(content);
        }
        self.cursor = cursor;
        self.anchor = self.cursor;
        self.value.update(cx, |s, cx| {
            *s = new_content;
            cx.notify();
        });
        self.reset_blink(cx);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_text_in_range(range_utf16, new_text, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let bounds = self.bounds?;
        let content = self.text(_cx);
        let start = offset_from_utf16(&content, range_utf16.start);
        let end = offset_from_utf16(&content, range_utf16.end);
        let line_height = _window.line_height();
        let display = if self.secret {
            "•".repeat(content.chars().count())
        } else {
            content.clone()
        };
        let offset = if self.secret {
            content[..start].chars().count() * "•".len()
        } else {
            start
        };
        let lines =
            shape_display_lines(&display, self.multiline.then_some(self.wrap_width), _window);
        let line_index = line_index_for_offset(&lines, &display, offset);
        let (shaped, line_start) = &lines[line_index];
        let x = shaped.x_for_index(offset.saturating_sub(*line_start));
        let width = if start == end {
            px(1.)
        } else {
            let end_offset = if self.secret {
                content[..end].chars().count() * "•".len()
            } else {
                end
            };
            if lines
                .get(line_index + 1)
                .is_some_and(|(_, start)| end_offset > *start)
            {
                px(1.)
            } else {
                (shaped.x_for_index(end_offset.saturating_sub(*line_start)) - x).max(px(1.))
            }
        };
        Some(Bounds::new(
            point(
                bounds.left() + x,
                bounds.top() + line_height * line_index as f32,
            ),
            size(width, line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        let content = self.text(cx);
        let display = if self.secret {
            "•".repeat(content.chars().count())
        } else {
            content.clone()
        };
        let line_height = window.line_height();
        let bounds = self.bounds?;
        let point = point - bounds.origin;
        let point_x = point.x
            + if self.multiline {
                px(0.)
            } else {
                self.scroll_x
            };
        let lines =
            shape_display_lines(&display, self.multiline.then_some(self.wrap_width), window);
        let row = ((point.y / line_height).floor() as usize).min(lines.len().saturating_sub(1));
        if let Some((shaped, start)) = lines.get(row) {
            let offset = *start + shaped.closest_index_for_x(point_x.max(px(0.)));
            if self.secret {
                let chars = display[..offset].chars().count();
                let byte = content
                    .char_indices()
                    .nth(chars)
                    .map_or(content.len(), |(byte, _)| byte);
                return Some(offset_to_utf16(&content, byte));
            }
            return Some(offset_to_utf16(&content, offset));
        }
        Some(content.len())
    }
}

impl gpui::Render for Editor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Editor>) -> impl IntoElement {
        EditorText {
            editor: cx.entity(),
        }
    }
}

struct EditorText {
    editor: Entity<Editor>,
}

struct EditorTextPrepaint {
    lines: Vec<(ShapedLine, usize)>,
    cursor: Option<PaintQuad>,
    selection: Vec<PaintQuad>,
    scroll_x: Pixels,
}

impl IntoElement for EditorText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for EditorText {
    type RequestLayoutState = ();
    type PrepaintState = EditorTextPrepaint;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let editor = self.editor.read(cx);
        let content = editor.value.read(cx);
        let display = if editor.secret {
            "•".repeat(content.chars().count())
        } else {
            content.clone()
        };
        let lines = shape_display_lines(
            &display,
            editor.multiline.then_some(editor.wrap_width),
            window,
        );
        let line_height = window.line_height();
        let mut style = gpui::Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (line_height * lines.len().max(1) as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let editor = self.editor.read(cx);
        let content = editor.value.read(cx).clone();
        let cursor_offset = editor.cursor;
        let display_content = if editor.secret {
            "•".repeat(content.chars().count())
        } else {
            content.clone()
        };
        let display_cursor_offset = if editor.secret {
            content[..cursor_offset].chars().count() * "•".len()
        } else {
            cursor_offset
        };
        let cursor_visible = editor.cursor_visible;
        let is_focused = editor.focus_handle.is_focused(window);
        let (selection_start, selection_end) = (
            editor.anchor.min(editor.cursor),
            editor.anchor.max(editor.cursor),
        );

        let style = window.text_style();
        let text_color = style.color;
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();

        let is_placeholder = content.is_empty();

        let lines: Vec<(ShapedLine, usize)> = if is_placeholder {
            let placeholder = editor.placeholder.clone();
            let run = TextRun {
                len: placeholder.len(),
                font: style.font(),
                color: hsla(0., 0., 0.5, 0.5),
                background_color: None,
                underline: None,
                strikethrough: None,
                letter_spacing: None,
            };
            vec![(
                window
                    .text_system()
                    .shape_line(placeholder, font_size, &[run], None),
                0,
            )]
        } else {
            shape_display_lines(
                &display_content,
                editor.multiline.then_some(editor.wrap_width),
                window,
            )
        };

        let mut scroll_x = if editor.multiline {
            px(0.)
        } else {
            editor.scroll_x
        };
        if !editor.multiline && !is_placeholder {
            let row = line_index_for_offset(&lines, &display_content, display_cursor_offset);
            let (line, start) = &lines[row];
            let x = line.x_for_index(display_cursor_offset.saturating_sub(*start));
            if x - scroll_x > bounds.size.width - px(2.) {
                scroll_x = x - bounds.size.width + px(2.);
            }
            if x < scroll_x {
                scroll_x = x;
            }
        }

        let cursor = if is_focused && cursor_visible && !is_placeholder {
            let cursor_line =
                line_index_for_offset(&lines, &display_content, display_cursor_offset);
            let (line, start) = &lines[cursor_line];
            let cursor_x =
                line.x_for_index(display_cursor_offset.saturating_sub(*start)) - scroll_x;
            Some(fill(
                Bounds::new(
                    point(
                        bounds.left() + cursor_x,
                        bounds.top() + line_height * cursor_line as f32,
                    ),
                    size(px(1.5), line_height),
                ),
                text_color,
            ))
        } else if is_focused && cursor_visible && is_placeholder {
            Some(fill(
                Bounds::new(
                    point(bounds.left(), bounds.top()),
                    size(px(1.5), line_height),
                ),
                text_color,
            ))
        } else {
            None
        };

        let (selection_start, selection_end) = if editor.secret {
            (
                content[..selection_start].chars().count() * "•".len(),
                content[..selection_end].chars().count() * "•".len(),
            )
        } else {
            (selection_start, selection_end)
        };
        let mut selection = Vec::new();
        if is_focused && selection_start < selection_end {
            for (line_index, (line, line_start)) in lines.iter().enumerate() {
                let line_end = *line_start + line.len();
                let start = selection_start.max(*line_start).min(line_end) - *line_start;
                let end = selection_end.min(line_end).max(*line_start) - *line_start;
                let selected_newline = display_content.as_bytes().get(line_end) == Some(&b'\n')
                    && selection_start <= line_end
                    && selection_end > line_end;
                if start < end || selected_newline {
                    let x0 = line.x_for_index(start) - scroll_x;
                    let x1 = if start < end {
                        line.x_for_index(end) - scroll_x
                    } else {
                        x0 + px(5.)
                    };
                    selection.push(fill(
                        Bounds::new(
                            point(
                                bounds.left() + x0,
                                bounds.top() + line_height * line_index as f32,
                            ),
                            size((x1 - x0).max(px(2.)), line_height),
                        ),
                        hsla(220. / 360., 0.8, 0.5, 0.35),
                    ));
                }
            }
        }
        EditorTextPrepaint {
            lines,
            cursor,
            selection,
            scroll_x,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.editor.update(cx, |editor, _| {
            editor.bounds = Some(bounds);
            editor.scroll_x = prepaint.scroll_x;
            editor.focus_handle.clone()
        });
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.editor.clone()),
            cx,
        );

        let line_height = window.line_height();
        for quad in prepaint.selection.drain(..) {
            window.paint_quad(quad);
        }
        for (i, (line, _)) in prepaint.lines.iter().enumerate() {
            let origin = point(
                bounds.left() - prepaint.scroll_x,
                bounds.top() + line_height * i as f32,
            );
            line.paint(origin, line_height, gpui::TextAlign::Left, None, window, cx)
                .unwrap();
        }

        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }
    }
}

fn shape_display_lines(
    content: &str,
    wrap_width: Option<Pixels>,
    window: &mut Window,
) -> Vec<(ShapedLine, usize)> {
    let style = window.text_style();
    let font_size = style.font_size.to_pixels(window.rem_size());
    let font = style.font();
    let mut result = Vec::new();
    let mut base = 0;
    for paragraph in content.split('\n') {
        let run = TextRun {
            len: paragraph.len(),
            font: font.clone(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
            letter_spacing: None,
        };
        let shared = SharedString::from(paragraph.to_string());
        let full = window
            .text_system()
            .shape_line(shared, font_size, &[run.clone()], None);
        if paragraph.is_empty() || wrap_width.is_none_or(|width| full.width() <= width) {
            result.push((full, base));
        } else {
            let width = wrap_width.unwrap();
            let boundaries = paragraph
                .grapheme_indices(true)
                .map(|(i, g)| i + g.len())
                .collect::<Vec<_>>();
            let mut start = 0;
            while start < paragraph.len() {
                let mut end = boundaries
                    .iter()
                    .copied()
                    .take_while(|&end| {
                        end > start && full.x_for_index(end) - full.x_for_index(start) <= width
                    })
                    .last()
                    .unwrap_or_else(|| {
                        boundaries.iter().copied().find(|&end| end > start).unwrap()
                    });
                if end < paragraph.len() {
                    if let Some(space_end) = boundaries
                        .iter()
                        .copied()
                        .take_while(|&boundary| boundary <= end)
                        .filter(|&boundary| {
                            paragraph[start..boundary]
                                .chars()
                                .next_back()
                                .is_some_and(char::is_whitespace)
                        })
                        .last()
                    {
                        if space_end > start {
                            end = space_end;
                        }
                    }
                }
                let piece = &paragraph[start..end];
                let text = SharedString::from(piece.to_string());
                let run = TextRun {
                    len: text.len(),
                    font: font.clone(),
                    color: style.color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                    letter_spacing: None,
                };
                result.push((
                    window
                        .text_system()
                        .shape_line(text, font_size, &[run], None),
                    base + start,
                ));
                start = end;
            }
        }
        base += paragraph.len() + 1;
    }
    result
}

fn line_index_for_offset(lines: &[(ShapedLine, usize)], text: &str, offset: usize) -> usize {
    lines
        .iter()
        .position(|(line, start)| {
            let end = *start + line.len();
            offset >= *start
                && (offset < end
                    || (offset == end
                        && (end == text.len() || text.as_bytes().get(end) == Some(&b'\n'))))
        })
        .unwrap_or_else(|| lines.len().saturating_sub(1))
}

pub(super) fn visual_line_count(
    content: &str,
    secret: bool,
    multiline: bool,
    width: Pixels,
    window: &mut Window,
) -> usize {
    let display = if secret {
        "•".repeat(content.chars().count())
    } else {
        content.to_string()
    };
    shape_display_lines(&display, multiline.then_some(width), window)
        .len()
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::{
        ceil_grapheme_boundary, floor_grapheme_boundary, format_digits, next_boundary,
        offset_from_utf16, offset_to_utf16, previous_boundary,
    };

    #[test]
    fn utf16_offsets_round_trip_non_ascii_text() {
        let text = "A😀é";
        for byte_offset in [0, 1, 5, 7] {
            let utf16 = offset_to_utf16(text, byte_offset);
            assert_eq!(offset_from_utf16(text, utf16), byte_offset);
        }
    }

    #[test]
    fn cursor_steps_by_grapheme() {
        let text = "aé😀";
        assert_eq!(next_boundary(text, 0), 1);
        assert_eq!(next_boundary(text, 1), 3);
        assert_eq!(previous_boundary(text, 7), 3);
    }

    #[test]
    fn date_mask_formats_digits_and_tracks_digit_cursor() {
        assert_eq!(format_digits("dd/mm/yyyy", "123", 3), ("12/3".into(), 4));
        assert_eq!(
            format_digits("dd/mm/yyyy", "1204198512", 10),
            ("12/04/1985".into(), 10)
        );
        assert_eq!(
            format_digits("dd/mm/yyyy HH:MM", "201020261615", 12),
            ("20/10/2026 16:15".into(), 16)
        );
    }

    #[test]
    fn utf16_ranges_expand_to_grapheme_boundaries() {
        let text = "e\u{301}x";
        assert_eq!(floor_grapheme_boundary(text, 2), 0);
        assert_eq!(ceil_grapheme_boundary(text, 2), 3);
        assert_eq!(floor_grapheme_boundary(text, text.len()), text.len());
    }
}

pub fn standard_actions<E: InteractiveElement>(editor: Entity<Editor>) -> impl FnOnce(E) -> E {
    move |element| {
        element
            .on_action({
                let editor = editor.clone();
                move |a: &Left, window, cx| editor.update(cx, |e, cx| e.left(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Right, window, cx| editor.update(cx, |e, cx| e.right(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Home, window, cx| editor.update(cx, |e, cx| e.home(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &End, window, cx| editor.update(cx, |e, cx| e.end(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Backspace, window, cx| {
                    editor.update(cx, |e, cx| e.backspace(a, window, cx))
                }
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Delete, window, cx| editor.update(cx, |e, cx| e.delete(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &SelectLeft, window, cx| {
                    editor.update(cx, |e, cx| e.select_left(a, window, cx))
                }
            })
            .on_action({
                let editor = editor.clone();
                move |a: &SelectRight, window, cx| {
                    editor.update(cx, |e, cx| e.select_right(a, window, cx))
                }
            })
            .on_action({
                let editor = editor.clone();
                move |a: &SelectAll, window, cx| {
                    editor.update(cx, |e, cx| e.select_all(a, window, cx))
                }
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Copy, window, cx| editor.update(cx, |e, cx| e.copy(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Cut, window, cx| editor.update(cx, |e, cx| e.cut(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Paste, window, cx| editor.update(cx, |e, cx| e.paste(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Undo, window, cx| editor.update(cx, |e, cx| e.undo(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Redo, window, cx| editor.update(cx, |e, cx| e.redo(a, window, cx))
            })
            .on_action({
                let editor = editor.clone();
                move |a: &WordLeft, window, cx| {
                    editor.update(cx, |e, cx| e.word_left(a, window, cx))
                }
            })
            .on_action({
                let editor = editor.clone();
                move |a: &WordRight, window, cx| {
                    editor.update(cx, |e, cx| e.word_right(a, window, cx))
                }
            })
            .on_action({
                let editor = editor.clone();
                move |a: &Up, window, cx| editor.update(cx, |e, cx| e.up(a, window, cx))
            })
            .on_action(move |a: &Down, window, cx| editor.update(cx, |e, cx| e.down(a, window, cx)))
    }
}
