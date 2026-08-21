mod app;
mod camera;
mod ui;

use app::CellSight;
use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                focus: true,
                ..Default::default()
            },
            |_, cx| {
                let view = cx.new(|_| CellSight::new());
                view.update(cx, |view, cx| view.start_camera(cx));
                view
            },
        )
        .unwrap();
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
