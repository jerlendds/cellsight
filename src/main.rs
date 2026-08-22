mod app;
mod camera;
mod ui;

use app::CellSight;
use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;
use std::borrow::Cow;

fn main() {
    application().run(|cx: &mut App| {
        cx.text_system()
            .add_fonts(vec![
                Cow::Borrowed(&include_bytes!("fonts/Neometric-Regular.otf")[..]),
                Cow::Borrowed(&include_bytes!("fonts/Neometric Medium (Regular).otf")[..]),
                Cow::Borrowed(&include_bytes!("fonts/Neometric Extra Bold (Bold).otf")[..]),
            ])
            .expect("bundled Neometric fonts should load");

        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(|_| CellSight::new()),
        )
        .unwrap();
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
