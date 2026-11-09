extern crate gpui_kit as gpui;

#[path = "../focus.rs"]
mod focus;
#[path = "../gpui_access.rs"]
mod gpui_access;
#[path = "../gpui_kit_input.rs"]
mod gpui_input;
#[path = "../gpui_ui/common.rs"]
mod gpui_primitives;
#[path = "../gpui_kit_ui.rs"]
mod gpui_ui;
#[path = "../patient.rs"]
mod patient;
#[path = "../storage/mod.rs"]
mod storage;

fn main() {
    gpui::application()
        .with_quit_mode(gpui::QuitMode::LastWindowClosed)
        .run(|cx| {
            gpui::init(cx);
            cx.bind_keys([
                gpui::KeyBinding::new("cmd-q", Quit, None),
                gpui::KeyBinding::new("ctrl-q", Quit, None),
                gpui::KeyBinding::new("tab", gpui_access::NextField, None),
                gpui::KeyBinding::new("shift-tab", gpui_access::PreviousField, None),
            ]);
            cx.on_action(|_: &Quit, cx| cx.quit());
            let options = gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds {
                    origin: gpui::Point::default(),
                    size: gpui::size(gpui::px(980.), gpui::px(840.)),
                })),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("KinéBilan 0.8.2 — GPUI Kit".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            gpui::open_window(options, cx, |_, cx| {
                use gpui::AppContext;
                cx.new(gpui_access::AccessView::new)
            })
            .expect("Failed to open KinéBilan");
            cx.activate(true);
        });
}

gpui::actions!(gpui_kit_preview, [Quit]);
