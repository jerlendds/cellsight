use super::theme::{ACCENT, BORDER, MUTED};
use crate::app::{CellSight, Tool};
use gpui::{
    Context, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, PathBuilder, canvas, div,
    img, prelude::*, px, rgb,
};

pub(crate) fn render(app: &CellSight, cx: &mut Context<CellSight>) -> impl IntoElement + use<> {
    let strokes = app.strokes.clone();
    let fps = format!("{} fps", app.fps_values[app.fps]);
    let mut viewport = div()
        .id("annotation-layer")
        .relative()
        .flex_1()
        .h_full()
        .overflow_hidden()
        .cursor_crosshair()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, e: &MouseDownEvent, _, cx| {
                if this.tool == Tool::Pointer {
                    return;
                }
                this.drawing = true;
                this.strokes.push(vec![e.position]);
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
            if this.drawing {
                if let Some(line) = this.strokes.last_mut() {
                    line.push(e.position);
                }
                cx.notify();
            }
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _, _, _| this.drawing = false),
        )
        .child(div().absolute().inset_0().bg(rgb(0x090c0f)));
    if let Some(frame) = app.camera_frame.clone() {
        viewport = viewport.child(img(frame).absolute().inset_0().size_full());
    } else {
        let status = app.camera_error.clone().unwrap_or_else(|| {
            if app.streaming {
                "Opening camera…".into()
            } else {
                "Connect a camera to begin".into()
            }
        });
        viewport = viewport.child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(MUTED))
                .child(status),
        );
    }
    viewport
        .child(
            canvas(
                move |_, _, _| {},
                move |_, _, window, _| {
                    for line in &strokes {
                        if line.len() < 2 {
                            continue;
                        }
                        let mut path = PathBuilder::stroke(px(2.));
                        path.move_to(line[0]);
                        for point in line.iter().skip(1) {
                            path.line_to(*point);
                        }
                        if let Ok(path) = path.build() {
                            window.paint_path(path, rgb(ACCENT));
                        }
                    }
                },
            )
            .absolute()
            .inset_0(),
        )
        .child(
            div()
                .absolute()
                .top_4()
                .left_4()
                .px_3()
                .py_2()
                .rounded_md()
                .bg(rgb(0x11151a))
                .border_1()
                .border_color(rgb(BORDER))
                .text_xs()
                .child(format!("{}  •  {}", app.resolutions[app.resolution], fps)),
        )
        .child(
            div()
                .absolute()
                .bottom_4()
                .left_4()
                .right_4()
                .flex()
                .justify_between()
                .text_xs()
                .text_color(rgb(MUTED))
                .child("Draw directly on the live view to annotate")
                .child(format!(
                    "{} annotation{}",
                    app.strokes.len(),
                    if app.strokes.len() == 1 { "" } else { "s" }
                )),
        )
}
