#![windows_subsystem = "windows"]

mod app;
mod focus;
mod patient;
mod storage;
mod ui;

use std::time::{Duration, Instant};

use sdl3::event::Event;
use sdl3::keyboard::Mod;
use sdl3::mouse::{Cursor, MouseButton, SystemCursor};
use sdl3::render::BlendMode;

use app::App;
use ui::{Caret, Fonts, Gui, Input, Rect};

const WHEEL_STEP_PX: f32 = 40.0;
const IDLE_REDRAW: Duration = Duration::from_millis(250);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    let ttf = sdl3::ttf::init()?;

    let window = video
        .window("KinéBilan", 980, 840)
        .position_centered()
        .resizable()
        .build()?;
    video.text_input().start(&window);
    let clipboard = video.clipboard();

    let mut canvas = window.into_canvas();
    canvas.set_blend_mode(BlendMode::Blend);
    let textures = canvas.texture_creator();
    let fonts = Fonts::load(&ttf)?;
    let mut event_pump = sdl.event_pump()?;

    let mut app = App::new();
    let mut started = Instant::now();
    let mut mouse = (0, 0);
    let mut mouse_down = false;
    let mut focus = None;
    let mut caret = Caret::default();
    let mut redraw_now = false;
    let arrow = Cursor::from_system(SystemCursor::Arrow)?;
    let hand = Cursor::from_system(SystemCursor::Hand)?;
    let ibeam = Cursor::from_system(SystemCursor::IBeam)?;

    'running: loop {
        let timeout = if redraw_now { Duration::ZERO } else { IDLE_REDRAW };
        let mut input = Input::default();

        let first = event_pump.wait_event_timeout(timeout);
        for event in first.into_iter().chain(event_pump.poll_iter()) {
            match event {
                Event::Quit { .. } => break 'running,
                Event::MouseMotion { x, y, .. } => mouse = (x as i32, y as i32),
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, clicks, .. } => {
                    mouse = (x as i32, y as i32);
                    mouse_down = true;
                    input.click = Some(mouse);
                    input.clicks = clicks;
                }
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => mouse_down = false,
                Event::MouseWheel { y, .. } => input.scroll_px -= y * WHEEL_STEP_PX,
                Event::TextInput { text, .. } => input.text.push_str(&text),
                Event::KeyDown { keycode: Some(key), keymod, .. } => {
                    input.shift = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
                    input.ctrl = keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD);
                    input.keys.push(key);
                }
                _ => {}
            }
        }
        input.mouse = mouse;
        input.mouse_down = mouse_down;
        input.caret_on = (started.elapsed().as_millis() / 500).is_multiple_of(2);

        let (width, height) = canvas.output_size()?;
        let screen = Rect::new(0, 0, width as i32, height as i32);
        let (next_focus, changed, mut next_caret, cursor) = {
            let mut gui = Gui::new(&mut canvas, &textures, &clipboard, input, focus, caret);
            app.draw(&mut gui, &fonts, screen);
            gui.finish();
            (gui.focus, gui.changed, gui.caret, gui.cursor)
        };
        match cursor {
            SystemCursor::Hand => hand.set(),
            SystemCursor::IBeam => ibeam.set(),
            _ => arrow.set(),
        }
        canvas.present();
        if next_caret.take_blink_reset() {
            started = Instant::now();
        }
        focus = next_focus;
        caret = next_caret;
        redraw_now = changed;
        if app.quit_requested() {
            break 'running;
        }
    }

    Ok(())
}
