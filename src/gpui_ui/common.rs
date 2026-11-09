use gpui::{IntoElement, SharedString, div, prelude::*, rgb};

#[derive(Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
    DangerConfirm,
    Outline,
}

pub fn card(title: impl Into<SharedString>, content: impl IntoElement) -> impl IntoElement {
    let title = title.into();
    let plain = title.is_empty() || title == "__full__";
    let full = title == "__full__";
    div()
        .flex()
        .flex_col()
        .gap_3()
        .when(full, |card| card.h_full())
        .when(!plain, |card| {
            card.p_6()
                .border_1()
                .border_color(rgb(0xe5e7eb))
                .bg(rgb(0xffffff))
        })
        .when(!plain, |card| {
            card.child(
                div()
                    .text_size(gpui::px(20.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0x111827))
                    .child(title.clone()),
            )
        })
        .child(content)
}
