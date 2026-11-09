use gpui::{App, ClickEvent, IntoElement, Pixels, SharedString, Window, div, prelude::*};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    form::Field,
    scroll::Scrollbar,
};

pub use crate::gpui_primitives::{ButtonStyle, card};

pub fn scrollable(
    id: &'static str,
    handle: &gpui::ScrollHandle,
    content: gpui::Div,
) -> impl IntoElement {
    div()
        .relative()
        .flex_1()
        .min_h_0()
        .child(
            content
                .id(id)
                .size_full()
                .overflow_y_scroll()
                .track_scroll(handle),
        )
        .child(Scrollbar::vertical(handle))
}

pub fn form_field(
    label: impl Into<SharedString>,
    control: impl IntoElement,
    error: Option<SharedString>,
) -> impl IntoElement {
    let label = label.into();
    div().id(label.clone()).flex_1().child(
        Field::new()
            .label(label)
            .child(control)
            .when_some(error, |field, error| field.description(error)),
    )
}

pub fn form_field_bold(
    label: impl Into<SharedString>,
    control: impl IntoElement,
) -> impl IntoElement {
    form_field(label, control, None).into_any_element()
}

pub fn button(
    id: &'static str,
    label: impl Into<SharedString>,
    style: ButtonStyle,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Button {
    let button = Button::new(id)
        .label(label)
        .tab_stop(false)
        .on_click(on_click);
    match style {
        ButtonStyle::Primary => button.primary(),
        ButtonStyle::Secondary => button,
        ButtonStyle::Danger | ButtonStyle::DangerConfirm => button.danger(),
        ButtonStyle::Outline => button.outline(),
    }
}

pub fn sized_button(
    id: &'static str,
    label: impl Into<SharedString>,
    style: ButtonStyle,
    width: Pixels,
    height: Pixels,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    button(id, label, style, on_click).w(width).h(height)
}
