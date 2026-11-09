//! `Input` — a single-line text input. The shaping layer over `Editor`.
//!
//! A text field backed by an app-owned string entity.

use gpui::{
    App, CursorStyle, Entity, EntityId, IntoElement, Pixels, SharedString, StyleRefinement, Window,
    div, hsla, prelude::*, px, white,
};

use super::editor::{Editor, standard_actions, visual_line_count};

#[derive(IntoElement)]
pub struct Input {
    id: &'static str,
    value: Entity<String>,
    fill: bool,
    focusable: bool,
    height: Option<Pixels>,
    measure_width: Option<Pixels>,
    width: Option<Pixels>,
    tab_order: Option<isize>,
    placeholder: SharedString,
    secret: bool,
    autofocus: bool,
    multiline: bool,
    mask: Option<&'static str>,
}

impl Input {
    /// Backed by a bare string; the editor is allocated internally.
    pub fn new(id: &'static str, value: Entity<String>) -> Self {
        Self {
            id,
            value,
            fill: false,
            focusable: true,
            height: None,
            measure_width: None,
            width: None,
            tab_order: None,
            placeholder: "Saisissez du texte…".into(),
            secret: false,
            autofocus: false,
            multiline: false,
            mask: None,
        }
    }

    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn measure_width(mut self, width: Pixels) -> Self {
        self.measure_width = Some(width);
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn tab_order(mut self, order: isize) -> Self {
        self.tab_order = Some(order);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn secret(mut self) -> Self {
        self.secret = true;
        self
    }

    pub fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn date(mut self) -> Self {
        self.mask = Some("dd/mm/yyyy");
        self
    }

    pub fn datetime(mut self) -> Self {
        self.mask = Some("dd/mm/yyyy HH:MM");
        self
    }
}

impl gpui::View for Input {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.value.entity_id())
    }

    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let value = self.value.clone();
        let placeholder = self.placeholder.clone();
        let secret = self.secret;
        let multiline = self.multiline;
        let mask = self.mask;
        let fill = self.fill;
        let focusable = self.focusable;
        let tab_order = self.tab_order;
        let input_height = self.height;
        let input_width = self.width.unwrap_or(self.measure_width.unwrap_or(px(400.)));
        let box_width = self.measure_width.unwrap_or(px(400.));
        let text_width = (box_width - px(16.)).max(px(1.));
        let editor = window.use_state(cx, move |window, cx| {
            Editor::over(
                value,
                placeholder,
                secret,
                multiline,
                mask,
                text_width,
                window,
                cx,
            )
        });
        let did_autofocus = window.use_state(cx, |_, _| false);

        let focus_handle = editor.read(cx).focus_handle.clone();
        if self.autofocus && !*did_autofocus.read(cx) {
            window.focus(&focus_handle, cx);
            did_autofocus.update(cx, |focused, _| *focused = true);
        }
        let is_focused = focus_handle.is_focused(window);
        let text_color = hsla(0., 0., 0.1, 1.);
        let height = if self.multiline {
            px(18.
                * visual_line_count(self.value.read(cx), secret, multiline, text_width, window)
                    .max(3) as f32
                + 16.)
        } else {
            px(36.)
        };

        let border = if is_focused {
            hsla(220. / 360., 0.8, 0.5, 1.)
        } else {
            hsla(0., 0., 0.75, 1.)
        };

        let input = div()
            .id(id)
            .key_context(if self.multiline {
                "TextArea"
            } else {
                "TextInput"
            })
            .track_focus(&focus_handle)
            .when_some(tab_order, |input, order| input.tab_index(order))
            .when(!focusable, |input| input.tab_index(-1))
            .cursor(CursorStyle::IBeam)
            .map(standard_actions(editor.clone()))
            .when(multiline, |this| {
                let editor = editor.clone();
                this.on_action(move |action: &super::Newline, window, cx| {
                    editor.update(cx, |editor, cx| editor.newline(action, window, cx));
                })
            })
            .w(input_width)
            .h(input_height.unwrap_or(if self.multiline { height } else { px(40.) }))
            .px(px(8.))
            .bg(white())
            .border_1()
            .border_color(border)
            .rounded(px(0.))
            .overflow_hidden()
            .flex()
            .when(multiline, |input| input.items_start().pt(px(8.)))
            .when(!multiline, |input| {
                input.flex_col().items_start().justify_center()
            })
            .line_height(px(20.))
            .text_size(px(14.))
            .font_family("Arial")
            .text_color(text_color)
            .child(if multiline {
                editor.cached(StyleRefinement::default()).into_any_element()
            } else {
                div()
                    .w_full()
                    .h(px(20.))
                    .child(editor.cached(StyleRefinement::default()))
                    .into_any_element()
            });
        if fill { input.w_full() } else { input }
    }
}
