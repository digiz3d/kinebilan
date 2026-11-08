use gpui::{
    App, ClickEvent, IntoElement, SharedString, StatefulInteractiveElement, Window, div,
    prelude::*, rgb,
};

fn scroll_moved(before: gpui::Pixels, after: gpui::Pixels, max: gpui::Pixels) -> bool {
    before.clamp(-max, gpui::px(0.)) != after.clamp(-max, gpui::px(0.))
}

fn scrollbar_thumb(viewport: f32, max_offset: f32, offset: f32) -> Option<(f32, f32)> {
    if max_offset <= 0. {
        return None;
    }
    let height = (viewport * viewport / (viewport + max_offset))
        .max(24.)
        .min(viewport);
    let top = (viewport - height) * (offset / max_offset).clamp(0., 1.);
    Some((top, height))
}

pub fn scrollable(
    id: &'static str,
    handle: &gpui::ScrollHandle,
    content: gpui::Div,
) -> impl IntoElement {
    let handle = handle.clone();
    let scroll_handle = handle.clone();
    let previous_offset = std::cell::Cell::new(handle.offset().y);
    div()
        .relative()
        .flex_1()
        .min_h_0()
        .on_scroll_wheel(move |_, _, cx| {
            // The child's native scroll handler runs before this wrapper's bubble handler.
            let mut offset = scroll_handle.offset();
            let max = scroll_handle.max_offset().y;
            offset.y = offset.y.clamp(-max, gpui::px(0.));
            let previous = previous_offset.replace(offset.y);
            scroll_handle.set_offset(offset);
            if scroll_moved(previous, offset.y, max) {
                cx.stop_propagation();
            }
        })
        .child(
            content
                .id(id)
                .size_full()
                .overflow_y_scroll()
                .track_scroll(&handle),
        )
        .child(
            gpui::canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let Some((top, height)) = scrollbar_thumb(
                        bounds.size.height.into(),
                        handle.max_offset().y.into(),
                        (-handle.offset().y).into(),
                    ) else {
                        return;
                    };
                    window.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            bounds.origin + gpui::point(gpui::px(0.), gpui::px(top)),
                            gpui::size(bounds.size.width, gpui::px(height)),
                        ),
                        rgb(0xd1d5db),
                    ));
                },
            )
            .absolute()
            .right(gpui::px(2.))
            .top(gpui::px(2.))
            .bottom(gpui::px(2.))
            .w(gpui::px(4.)),
        )
}

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

pub fn form_field(
    label: impl Into<SharedString>,
    control: impl IntoElement,
    error: Option<SharedString>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_size(gpui::px(13.))
                .font_weight(gpui::FontWeight::NORMAL)
                .text_color(rgb(0x374151))
                .child(label.into()),
        )
        .child(control)
        .when_some(error, |field, error| {
            field.child(div().text_sm().text_color(rgb(0xb91c1c)).child(error))
        })
}

impl ButtonStyle {
    fn colors(self) -> (u32, u32, u32, u32) {
        match self {
            Self::Primary => (0x2563eb, 0xffffff, 0x2563eb, 0x1d4ed8),
            Self::Secondary => (0xe5e7eb, 0x374151, 0xe5e7eb, 0xd1d5db),
            Self::Danger => (0xfee2e2, 0xb91c1c, 0xfca5a5, 0xfecaca),
            Self::DangerConfirm => (0xdc2626, 0xffffff, 0xb91c1c, 0xb91c1c),
            Self::Outline => (0xffffff, 0x2563eb, 0x2563eb, 0xeff6ff),
        }
    }
}

pub fn form_field_bold(
    label: impl Into<SharedString>,
    control: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .child(
            div()
                .text_size(gpui::px(13.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(0x374151))
                .child(label.into()),
        )
        .child(control)
}

pub fn button(
    id: &'static str,
    label: impl Into<SharedString>,
    style: ButtonStyle,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let label = label.into();
    let (background, foreground, border, hover) = style.colors();
    div()
        .id(id)
        .tab_index(-1)
        .px_3()
        .py_2()
        .rounded_md()
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(background))
        .text_color(rgb(foreground))
        .cursor_pointer()
        .hover(move |style| style.bg(rgb(hover)))
        .child(div().text_size(gpui::px(14.)).child(label))
        .on_click(on_click)
}

pub fn sized_button(
    id: &'static str,
    label: impl Into<SharedString>,
    style: ButtonStyle,
    width: gpui::Pixels,
    height: gpui::Pixels,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let label = label.into();
    let (background, foreground, border, hover) = style.colors();
    div()
        .id(id)
        .tab_index(-1)
        .w(width)
        .h(height)
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(background))
        .text_size(gpui::px(14.))
        .font_weight(gpui::FontWeight::NORMAL)
        .text_color(rgb(foreground))
        .cursor_pointer()
        .hover(move |style| style.bg(rgb(hover)))
        .child(label)
        .on_click(on_click)
}

#[cfg(test)]
mod tests {
    use super::{scroll_moved, scrollbar_thumb};

    #[test]
    fn scroll_chains_only_when_inner_viewport_cannot_move() {
        let moved =
            |before, after, max| scroll_moved(gpui::px(before), gpui::px(after), gpui::px(max));
        assert!(moved(-50., -60., 100.));
        assert!(moved(-50., -40., 100.));
        assert!(!moved(0., 10., 100.));
        assert!(!moved(-100., -110., 100.));
        assert!(moved(0., -10., 100.));
        assert!(moved(-100., -90., 100.));
        assert!(moved(-90., -110., 100.));
        assert!(!moved(-100., -120., 100.));
        assert!(!moved(0., -10., 0.));
    }

    #[test]
    fn scrollbar_hides_without_overflow_and_tracks_scroll_position() {
        assert_eq!(scrollbar_thumb(100., 0., 0.), None);
        assert_eq!(scrollbar_thumb(100., 100., 0.), Some((0., 50.)));
        assert_eq!(scrollbar_thumb(100., 100., 50.), Some((25., 50.)));
        assert_eq!(scrollbar_thumb(100., 100., 100.), Some((50., 50.)));
        assert_eq!(scrollbar_thumb(100., 900., 900.), Some((76., 24.)));
        assert_eq!(scrollbar_thumb(10., 90., 90.), Some((0., 10.)));
    }
}
