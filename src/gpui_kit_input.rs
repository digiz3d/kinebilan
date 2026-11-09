use gpui::{App, Entity, EntityId, IntoElement, Pixels, SharedString, Window, div, prelude::*, px};
use gpui_kit::component::input::{
    Input as KitInput, InputEvent, InputState, Textarea, TextareaState,
};

#[derive(IntoElement)]
pub struct Input {
    id: &'static str,
    value: Entity<String>,
    fill: bool,
    focusable: bool,
    height: Option<Pixels>,
    width: Pixels,
    tab_order: isize,
    placeholder: SharedString,
    secret: bool,
    autofocus: bool,
    multiline: bool,
    mask: Option<&'static str>,
}

impl Input {
    pub fn new(id: &'static str, value: Entity<String>) -> Self {
        Self {
            id,
            value,
            fill: false,
            focusable: true,
            height: None,
            width: px(400.),
            tab_order: 0,
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

    pub fn measure_width(self, width: Pixels) -> Self {
        self.width(width)
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn tab_order(mut self, order: isize) -> Self {
        self.tab_order = order;
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
        self.mask = Some("99/99/9999");
        self
    }

    pub fn datetime(mut self) -> Self {
        self.mask = Some("99/99/9999 99:99");
        self
    }
}

impl gpui::View for Input {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.value.entity_id())
    }

    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value = self.value.clone();
        let placeholder = self.placeholder;
        let autofocus = self.autofocus;
        let index = if self.focusable { self.tab_order } else { -1 };
        let control = if self.multiline {
            let state = window
                .use_state(cx, |window, cx| {
                    let state = cx.new(|cx| {
                        TextareaState::new(window, cx)
                            .placeholder(placeholder)
                            .default_value(value.read(cx).clone())
                            .auto_grow(3, 30)
                    });
                    cx.subscribe(&state, move |_, state, event, cx| {
                        if matches!(event, InputEvent::Change) {
                            let text = state.read(cx).value().to_string();
                            value.update(cx, |value, cx| {
                                *value = text;
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                    if autofocus {
                        state.update(cx, |state, cx| state.focus(window, cx));
                    }
                    state
                })
                .read(cx)
                .clone();
            let text = self.value.read(cx).clone();
            if state.read(cx).value().as_str() != text {
                state.update(cx, |state, cx| state.set_value(text, window, cx));
            }
            Textarea::new(&state)
                .tab_index(index)
                .disabled(!self.focusable)
                .when_some(self.height, |input, height| input.h(height))
                .into_any_element()
        } else {
            let state = window
                .use_state(cx, |window, cx| {
                    let state = cx.new(|cx| {
                        let state = InputState::new(window, cx)
                            .placeholder(placeholder)
                            .masked(self.secret);
                        let state = if let Some(mask) = self.mask {
                            state.mask_pattern(mask)
                        } else {
                            state
                        };
                        state.default_value(value.read(cx).clone())
                    });
                    cx.subscribe_in(&state, window, move |_, state, event, window, cx| {
                        if matches!(event, InputEvent::Change) {
                            let text = state.read(cx).value().to_string();
                            value.update(cx, |value, cx| {
                                *value = text;
                                cx.notify();
                            });
                        } else if self.secret && matches!(event, InputEvent::PressEnter { .. }) {
                            window.dispatch_action(Box::new(crate::gpui_access::Submit), cx);
                        }
                    })
                    .detach();
                    if autofocus {
                        state.update(cx, |state, cx| state.focus(window, cx));
                    }
                    state
                })
                .read(cx)
                .clone();
            let text = self.value.read(cx).clone();
            if state.read(cx).value().as_str() != text {
                state.update(cx, |state, cx| state.set_value(text, window, cx));
            }
            KitInput::new(&state)
                .tab_index(index)
                .disabled(!self.focusable)
                .h(self.height.unwrap_or(px(40.)))
                .into_any_element()
        };
        div()
            .id(self.id)
            // shortcut: Kit 0.9 ignores input tab_index; remove this group when Kit applies it.
            .tab_group()
            .tab_index(index)
            .tab_stop(false)
            .w(self.width)
            .when(self.fill, |input| input.w_full())
            .when_some(self.height, |input, height| input.h(height))
            .child(control)
    }
}

#[cfg(all(test, feature = "gpui-kit-tests"))]
mod tests {
    use super::Input;
    use gpui::{
        AppContext, Context, Entity, IntoElement, Render, TestAppContext, Window, div, prelude::*,
    };

    struct Form {
        first: Entity<String>,
        notes: Entity<String>,
    }

    impl Render for Form {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .child(crate::gpui_ui::form_field_bold(
                    "First",
                    Input::new("first", self.first.clone())
                        .autofocus()
                        .tab_order(1),
                ))
                .child(crate::gpui_ui::form_field_bold(
                    "Notes",
                    Input::new("notes", self.notes.clone())
                        .multiline()
                        .tab_order(2),
                ))
                .child(crate::gpui_ui::button(
                    "save",
                    "Save",
                    crate::gpui_ui::ButtonStyle::Primary,
                    |_, _, _| {},
                ))
                .on_action(|_: &crate::gpui_access::NextField, window, cx| window.focus_next(cx))
                .on_action(|_: &crate::gpui_access::PreviousField, window, cx| {
                    window.focus_prev(cx)
                })
        }
    }

    #[gpui_kit::test]
    fn kit_inputs_sync_and_wrap_focus(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            cx.bind_keys([
                gpui::KeyBinding::new("tab", crate::gpui_access::NextField, None),
                gpui::KeyBinding::new("shift-tab", crate::gpui_access::PreviousField, None),
            ]);
        });
        let (form, cx) = cx.add_window_view(|_, cx| Form {
            first: cx.new(|_| String::new()),
            notes: cx.new(|_| String::new()),
        });
        cx.simulate_input("Élodie");
        cx.simulate_keystrokes("tab");
        cx.simulate_input("Notes");
        cx.simulate_keystrokes("tab");
        cx.simulate_input("!");
        form.read_with(cx, |form, cx| {
            assert_eq!(form.first.read(cx), "Élodie!");
            assert_eq!(form.notes.read(cx), "Notes");
        });
        cx.simulate_keystrokes("shift-tab");
        cx.simulate_input(" suite");
        form.read_with(cx, |form, cx| {
            assert_eq!(form.notes.read(cx), "Notes suite")
        });

        form.update(cx, |form, cx| {
            form.first.update(cx, |value, cx| {
                *value = "Updated".into();
                cx.notify();
            });
            cx.notify();
        });
        cx.simulate_keystrokes("tab");
        cx.simulate_input("?");
        form.read_with(cx, |form, cx| assert_eq!(form.first.read(cx), "Updated?"));
    }

    struct Dates {
        date: Entity<String>,
        datetime: Entity<String>,
    }

    impl Render for Dates {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .child(Input::new("date", self.date.clone()).date().autofocus())
                .child(Input::new("datetime", self.datetime.clone()).datetime())
        }
    }

    #[gpui_kit::test]
    fn kit_date_masks_preserve_display_values(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (form, cx) = cx.add_window_view(|_, cx| Dates {
            date: cx.new(|_| String::new()),
            datetime: cx.new(|_| "08/11/2026 09:30".into()),
        });
        cx.simulate_input("12041985");
        form.read_with(cx, |form, cx| {
            assert_eq!(form.date.read(cx), "12/04/1985");
            assert_eq!(form.datetime.read(cx), "08/11/2026 09:30");
        });
    }

    struct Access {
        password: Entity<String>,
        submitted: Option<String>,
    }

    impl Render for Access {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(
                    Input::new("password", self.password.clone())
                        .secret()
                        .autofocus(),
                )
                .on_action(cx.listener(|this, _: &crate::gpui_access::Submit, _, cx| {
                    this.submitted = Some(this.password.read(cx).clone());
                    cx.notify();
                }))
        }
    }

    #[gpui_kit::test]
    fn kit_password_enter_submits_current_value(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (form, cx) = cx.add_window_view(|_, cx| Access {
            password: cx.new(|_| String::new()),
            submitted: None,
        });
        cx.simulate_input("password123");
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        form.read_with(cx, |form, _| {
            assert_eq!(form.submitted.as_deref(), Some("password123"))
        });
    }
}
