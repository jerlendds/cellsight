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
            cx.listener(|this, e: &MouseDownEvent, window, cx| {
                if this.selecting_focus_region {
                    let size = window.viewport_size();
                    let width: f32 = size.width.into();
                    let height: f32 = size.height.into();
                    let x: f32 = e.position.x.into();
                    let y: f32 = e.position.y.into();
                    let point = [
                        ((x - 310.0) / (width - 310.0).max(1.0)).clamp(0.0, 1.0),
                        ((y - 54.0) / (height - 54.0).max(1.0)).clamp(0.0, 1.0),
                    ];
                    this.focus_region_anchor = Some(point);
                    this.focus_region = Some([point[0], point[1], point[0], point[1]]);
                    cx.notify();
                    return;
                }
                if this.comparison_enabled
                    && this.comparison_left_frame.is_some()
                    && this.comparison_right_frame.is_some()
                {
                    let size = window.viewport_size();
                    let width: f32 = size.width.into();
                    let x: f32 = e.position.x.into();
                    let normalized = ((x - 310.0) / (width - 310.0).max(1.0)).clamp(0.0, 1.0);
                    if (normalized - this.comparison_split).abs() < 0.035 {
                        this.dragging_comparison = true;
                        cx.notify();
                        return;
                    }
                }
                if this.tool == Tool::Pointer {
                    return;
                }
                this.drawing = true;
                this.strokes.push(vec![e.position]);
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, window, cx| {
            if this.dragging_comparison {
                let size = window.viewport_size();
                let width: f32 = size.width.into();
                let x: f32 = e.position.x.into();
                this.comparison_split = ((x - 310.0) / (width - 310.0).max(1.0)).clamp(0.02, 0.98);
                cx.notify();
                return;
            }
            if let Some(anchor) = this.focus_region_anchor {
                let size = window.viewport_size();
                let width: f32 = size.width.into();
                let height: f32 = size.height.into();
                let x: f32 = e.position.x.into();
                let y: f32 = e.position.y.into();
                let point = [
                    ((x - 310.0) / (width - 310.0).max(1.0)).clamp(0.0, 1.0),
                    ((y - 54.0) / (height - 54.0).max(1.0)).clamp(0.0, 1.0),
                ];
                this.focus_region = Some([
                    anchor[0].min(point[0]),
                    anchor[1].min(point[1]),
                    anchor[0].max(point[0]),
                    anchor[1].max(point[1]),
                ]);
                cx.notify();
                return;
            }
            if this.drawing {
                if let Some(line) = this.strokes.last_mut() {
                    line.push(e.position);
                }
                cx.notify();
            }
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| {
                this.drawing = false;
                this.dragging_comparison = false;
                if this.focus_region_anchor.take().is_some() {
                    this.selecting_focus_region = false;
                    this.rerun_captured_sweep(cx);
                    cx.notify();
                }
            }),
        )
        .child(div().absolute().inset_0().bg(rgb(0x090c0f)));
    if app.comparison_enabled
        && let (Some(left), Some(right)) = (
            app.comparison_left_frame.clone(),
            app.comparison_right_frame.clone(),
        )
    {
        let split = app.comparison_split.clamp(0.02, 0.98);
        viewport = viewport
            .child(img(right).absolute().inset_0().size_full())
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(gpui::relative(split))
                    .overflow_hidden()
                    .child(
                        img(left)
                            .absolute()
                            .inset_0()
                            .w(gpui::relative(1.0 / split))
                            .h_full(),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(gpui::relative(split))
                    .w(px(2.))
                    .bg(rgb(ACCENT))
                    .child(
                        div()
                            .absolute()
                            .top(gpui::relative(0.5))
                            .left(px(-10.))
                            .w(px(22.))
                            .h(px(38.))
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(ACCENT))
                            .bg(rgb(0x11151a))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child("↔"),
                    ),
            );
    } else if let Some(frame) = app.focus_frame.clone() {
        viewport = viewport.child(img(frame).absolute().inset_0().size_full());
    } else if let Some(frame) = app.camera_frame.clone() {
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
    if let Some([left, top, right, bottom]) = app.focus_region {
        viewport = viewport.child(
            div()
                .absolute()
                .left(gpui::relative(left))
                .top(gpui::relative(top))
                .w(gpui::relative((right - left).max(0.002)))
                .h(gpui::relative((bottom - top).max(0.002)))
                .border_2()
                .border_color(rgb(ACCENT)),
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
                .child(if app.recording {
                    format!(
                        "Sweep focus steadily through Z · {} planes",
                        app.focus_sweep.len()
                    )
                } else if app.selecting_focus_region {
                    "Drag over the specimen to select the depth-analysis region".to_owned()
                } else if let Some(result) = &app.focus_result {
                    if result.plane_limited {
                        format!(
                            "Focus profile · {} planes · {} × {} · memory limit reached; select a smaller region for complete Z coverage",
                            result.planes, result.width, result.height
                        )
                    } else {
                        format!(
                            "Focus profile · {} planes · {} × {} analysis",
                            result.planes, result.width, result.height
                        )
                    }
                } else {
                    "Draw directly on the live view to annotate".to_owned()
                })
                .child(format!(
                    "{} annotation{}",
                    app.strokes.len(),
                    if app.strokes.len() == 1 { "" } else { "s" }
                )),
        )
}
