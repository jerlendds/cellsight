use crate::app::{Annotation, CellSight, Tool};
use cellsight_color_picker::{color_picker, parse_color};
use cellsight_theme::{ACCENT, BORDER, CANVAS, MUTED};
use gpui::{
    Animation, AnimationExt, Context, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, PathBuilder, Pixels, Point, ScrollDelta, ScrollWheelEvent, SharedString,
    canvas, div, ease_out_quint, img, point, prelude::*, px, rgb,
};
use std::time::Duration;

const MIN_ANNOTATION_SIZE: f32 = 1.0;
const MAX_ANNOTATION_SIZE: f32 = 24.0;

fn line_angle_degrees(start: Point<Pixels>, end: Point<Pixels>) -> u16 {
    let dx = f32::from(end.x) - f32::from(start.x);
    let dy = f32::from(end.y) - f32::from(start.y);
    (-dy).atan2(dx).to_degrees().rem_euclid(360.0).round() as u16 % 360
}

fn angle_label_position(start: Point<Pixels>, degrees: u16) -> (Pixels, Pixels) {
    let bisector = (degrees as f32).to_radians() / 2.0;
    let distance = 30.0;
    (
        px(f32::from(start.x) + distance * bisector.cos()),
        px(f32::from(start.y) - distance * bisector.sin()),
    )
}

fn snap_to_angle_increment(
    start: Point<Pixels>,
    target: Point<Pixels>,
    increment: f32,
) -> Point<Pixels> {
    let start_x: f32 = start.x.into();
    let start_y: f32 = start.y.into();
    let target_x: f32 = target.x.into();
    let target_y: f32 = target.y.into();
    let dx = target_x - start_x;
    let dy = target_y - start_y;
    let distance = (dx * dx + dy * dy).sqrt();
    if distance <= f32::EPSILON {
        return target;
    }

    let angle = (dy.atan2(dx) / increment).round() * increment;
    point(
        px(start_x + distance * angle.cos()),
        px(start_y + distance * angle.sin()),
    )
}

fn transform_annotation(annotation: &mut Annotation, scale: f32, rotation: f32) {
    if annotation.tool == Tool::Text {
        annotation.size = (annotation.size * scale).clamp(MIN_ANNOTATION_SIZE, MAX_ANNOTATION_SIZE);
        return;
    }
    if annotation.points.is_empty() {
        return;
    }
    let (sum_x, sum_y) = annotation.points.iter().fold((0.0, 0.0), |(x, y), point| {
        (x + f32::from(point.x), y + f32::from(point.y))
    });
    let count = annotation.points.len() as f32;
    let center_x = sum_x / count;
    let center_y = sum_y / count;
    let (sin, cos) = rotation.sin_cos();
    for point in &mut annotation.points {
        let x = (f32::from(point.x) - center_x) * scale;
        let y = (f32::from(point.y) - center_y) * scale;
        *point = gpui::point(
            px(center_x + x * cos - y * sin),
            px(center_y + x * sin + y * cos),
        );
    }
}

pub(crate) fn render(app: &CellSight, cx: &mut Context<CellSight>) -> impl IntoElement + use<> {
    let annotations = app.annotations.clone();
    let size_preview = app.annotation_size_preview.map(|position| {
        (
            position,
            app.tool,
            app.annotation_size,
            app.annotation_color,
            app.annotation_size_preview_generation,
        )
    });
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
                if !this.annotations_open {
                    return;
                }
                this.undone_annotations.clear();
                if this.tool == Tool::Text {
                    let index = this.annotations.len();
                    this.annotations.push(Annotation {
                        tool: Tool::Text,
                        color: this.annotation_color,
                        points: vec![e.position],
                        text: "".into(),
                        size: this.annotation_size,
                    });
                    this.editing_annotation = Some(index);
                    this.selected_annotation = Some(index);
                    this.drawing = false;
                    if let Some(focus) = &this.annotation_text_focus {
                        focus.focus(window, cx);
                    }
                    cx.notify();
                    return;
                }
                this.drawing = true;
                this.selected_annotation = Some(this.annotations.len());
                this.annotations.push(Annotation {
                    tool: this.tool,
                    color: this.annotation_color,
                    points: vec![e.position, e.position],
                    text: "".into(),
                    size: this.annotation_size,
                });
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
                if let Some(annotation) = this.annotations.last_mut() {
                    if annotation.tool == Tool::Pencil {
                        annotation.points.push(e.position);
                    } else {
                        let endpoint = if e.modifiers.shift {
                            snap_to_angle_increment(
                                annotation.points[0],
                                e.position,
                                std::f32::consts::PI / 12.0,
                            )
                        } else if annotation.tool == Tool::Angle {
                            snap_to_angle_increment(
                                annotation.points[0],
                                e.position,
                                std::f32::consts::PI / 180.0,
                            )
                        } else {
                            e.position
                        };
                        if let Some(current_endpoint) = annotation.points.last_mut() {
                            *current_endpoint = endpoint;
                        }
                    }
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
        .on_scroll_wheel(cx.listener(|this, e: &ScrollWheelEvent, _, cx| {
            if !this.annotations_open || !(e.modifiers.control || e.modifiers.platform) {
                return;
            }
            let delta = match e.delta {
                ScrollDelta::Pixels(delta) => f32::from(delta.y),
                ScrollDelta::Lines(delta) => delta.y * 20.0,
            };
            if delta.abs() <= f32::EPSILON {
                return;
            }
            let factor = if delta > 0.0 { 1.12 } else { 1.0 / 1.12 };
            this.annotation_size =
                (this.annotation_size * factor).clamp(MIN_ANNOTATION_SIZE, MAX_ANNOTATION_SIZE);
            this.annotation_size_preview = Some(e.position);
            this.annotation_size_preview_generation =
                this.annotation_size_preview_generation.wrapping_add(1);
            cx.stop_propagation();
            cx.notify();
        }))
        .child(div().absolute().inset_0().bg(rgb(CANVAS)));
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
                .ml_auto()
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
    viewport = viewport.child(
        canvas(
            move |_, _, _| {},
            move |_, _, window, _| {
                for annotation in &annotations {
                    if annotation.points.len() < 2 {
                        continue;
                    }
                    let mut path = PathBuilder::stroke(px(annotation.size));
                    path.move_to(annotation.points[0]);
                    match annotation.tool {
                        Tool::Text => continue,
                        Tool::Pencil => {
                            for point in annotation.points.iter().skip(1) {
                                path.line_to(*point);
                            }
                        }
                        Tool::Line => path.line_to(annotation.points[1]),
                        Tool::Angle => {
                            let start = annotation.points[0];
                            let end = annotation.points[1];
                            path.line_to(end);

                            let start_x = f32::from(start.x);
                            let start_y = f32::from(start.y);
                            let end_x = f32::from(end.x);
                            let end_y = f32::from(end.y);
                            let dx = end_x - start_x;
                            let dy = end_y - start_y;
                            let length = (dx * dx + dy * dy).sqrt();
                            if length > 0.5 {
                                let head_length = (annotation.size * 6.0).min(length * 0.4);
                                let line_angle = dy.atan2(dx);
                                let spread = std::f32::consts::FRAC_PI_6;
                                for wing_angle in [line_angle + spread, line_angle - spread] {
                                    path.move_to(end);
                                    path.line_to(point(
                                        px(end_x - head_length * wing_angle.cos()),
                                        px(end_y - head_length * wing_angle.sin()),
                                    ));
                                }
                            }

                            let degrees = line_angle_degrees(start, end);
                            let sweep = (degrees as f32).to_radians();
                            if sweep > f32::EPSILON {
                                let radius = (annotation.size * 8.0).clamp(16.0, 28.0);
                                let segments = ((degrees as usize + 7) / 8).max(2);
                                path.move_to(point(px(start_x + radius), px(start_y)));
                                for segment in 1..=segments {
                                    let angle = sweep * segment as f32 / segments as f32;
                                    path.line_to(point(
                                        px(start_x + radius * angle.cos()),
                                        px(start_y - radius * angle.sin()),
                                    ));
                                }
                            }
                        }
                        Tool::Arrow => {
                            let start = annotation.points[0];
                            let end = annotation.points[1];
                            path.line_to(end);

                            let start_x: f32 = start.x.into();
                            let start_y: f32 = start.y.into();
                            let end_x: f32 = end.x.into();
                            let end_y: f32 = end.y.into();
                            let dx = end_x - start_x;
                            let dy = end_y - start_y;
                            let length = (dx * dx + dy * dy).sqrt();
                            if length > 0.5 {
                                let head_length = (annotation.size * 6.0).min(length * 0.4);
                                let angle = dy.atan2(dx);
                                let spread = std::f32::consts::FRAC_PI_6;
                                for wing_angle in [angle + spread, angle - spread] {
                                    path.move_to(end);
                                    path.line_to(point(
                                        px(end_x - head_length * wing_angle.cos()),
                                        px(end_y - head_length * wing_angle.sin()),
                                    ));
                                }
                            }
                        }
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, rgb(annotation.color));
                    }
                }
            },
        )
        .absolute()
        .inset_0(),
    );

    for (index, annotation) in app.annotations.iter().enumerate() {
        if annotation.tool != Tool::Angle || annotation.points.len() < 2 {
            continue;
        }
        let start = annotation.points[0];
        let degrees = line_angle_degrees(start, annotation.points[1]);
        let (label_x, label_y) = angle_label_position(start, degrees);
        let label = format!("{degrees}°");
        viewport = viewport.child(
            div()
                .id(("angle-label", index))
                .absolute()
                .left(px((f32::from(label_x) - 310.0 - 10.0).max(0.0)))
                .top(px((f32::from(label_y) - 48.0 - 14.0).max(0.0)))
                .px_1()
                .text_xs()
                .child(
                    div()
                        .absolute()
                        .left(px(5.0))
                        .top(px(1.0))
                        .text_color(gpui::rgba(0x000000b3))
                        .child(label.clone()),
                )
                .child(div().text_color(rgb(annotation.color)).child(label)),
        );
    }

    for (index, annotation) in app.annotations.iter().enumerate() {
        if annotation.tool != Tool::Text || annotation.points.is_empty() {
            continue;
        }
        let position = annotation.points[0];
        let x: f32 = position.x.into();
        let y: f32 = position.y.into();
        let editing = app.editing_annotation == Some(index);
        let display_text: SharedString = if annotation.text.is_empty() && editing {
            "Type annotation…".into()
        } else {
            annotation.text.clone()
        };
        let text_focus = app.annotation_text_focus.clone();
        let focus_on_click = text_focus.clone();
        viewport = viewport.child(
            div()
                .id(("text-annotation", index))
                .absolute()
                .left(px((x - 310.0).max(0.0)))
                .top(px((y - 48.0).max(0.0)))
                .min_w(px(120.))
                .max_w(px(320.))
                .min_h(px(34.))
                .px_2()
                .flex()
                .items_center()
                .text_size(px((annotation.size * 7.0).clamp(10.0, 72.0)))
                .text_color(rgb(annotation.color))
                .cursor_text()
                .when_some(text_focus, |element, focus| element.track_focus(&focus))
                .child(display_text)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.editing_annotation = Some(index);
                        this.selected_annotation = Some(index);
                        this.object_color_picker_open = false;
                        if let Some(focus) = &focus_on_click {
                            focus.focus(window, cx);
                        }
                        cx.notify();
                    }),
                )
                .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                    if this.editing_annotation != Some(index)
                        || event.keystroke.modifiers.control
                        || event.keystroke.modifiers.platform
                    {
                        return;
                    }
                    if event.keystroke.key == "backspace" {
                        if let Some(annotation) = this.annotations.get_mut(index) {
                            let mut text = annotation.text.to_string();
                            text.pop();
                            annotation.text = text.into();
                        }
                    } else if event.keystroke.key == "enter" {
                        if event.keystroke.modifiers.shift {
                            if let Some(annotation) = this.annotations.get_mut(index) {
                                let mut text = annotation.text.to_string();
                                text.push('\n');
                                annotation.text = text.into();
                            }
                        } else {
                            this.editing_annotation = None;
                        }
                    } else if let Some(characters) = &event.keystroke.key_char
                        && !event.keystroke.modifiers.alt
                    {
                        if let Some(annotation) = this.annotations.get_mut(index) {
                            let mut text = annotation.text.to_string();
                            text.push_str(characters);
                            annotation.text = text.into();
                        }
                    }
                    cx.notify();
                })),
        );
    }

    if let Some(index) = app.selected_annotation
        && let Some(annotation) = app.annotations.get(index)
        && !annotation.points.is_empty()
    {
        let mut left = f32::INFINITY;
        let mut top = f32::INFINITY;
        let mut right = f32::NEG_INFINITY;
        let mut bottom = f32::NEG_INFINITY;
        for point in &annotation.points {
            let x = f32::from(point.x);
            let y = f32::from(point.y);
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        }
        if annotation.tool == Tool::Text {
            right = left + (annotation.text.len().max(8) as f32 * annotation.size * 3.5);
            bottom = top + (annotation.text.lines().count().max(1) as f32 * annotation.size * 9.0);
        }
        let padding = 8.0;
        let local_left = (left - 310.0 - padding).max(0.0);
        let local_top = (top - 48.0 - padding).max(0.0);
        let width = (right - left + padding * 2.0).max(28.0);
        let height = (bottom - top + padding * 2.0).max(28.0);
        let picker_focus = app.color_input_focus.clone();
        let selection_text_focus = app.annotation_text_focus.clone();
        let selected_tool = annotation.tool;
        if let Some(picker_focus) = picker_focus {
            viewport = viewport.child(
                div()
                    .absolute()
                    .left(px(local_left))
                    .top(px(local_top))
                    .w(px(width))
                    .h(px(height))
                    .border_1()
                    .border_color(rgb(ACCENT))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.selected_annotation = Some(index);
                            if selected_tool == Tool::Text {
                                this.editing_annotation = Some(index);
                                if let Some(focus) = &selection_text_focus {
                                    focus.focus(window, cx);
                                }
                            }
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(-3.))
                            .top(px(-3.))
                            .size(px(6.))
                            .bg(rgb(ACCENT)),
                    )
                    .child(
                        div()
                            .absolute()
                            .right(px(-3.))
                            .bottom(px(-3.))
                            .size(px(6.))
                            .bg(rgb(ACCENT)),
                    )
                    .child(
                        div()
                            .absolute()
                            .right_0()
                            .top(px(-42.))
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id(("annotation-scale-down", index))
                                    .size(px(30.))
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(BORDER))
                                    .bg(rgb(0x11151a))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .child("−")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(annotation) = this.annotations.get_mut(index) {
                                            transform_annotation(annotation, 0.9, 0.0);
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .id(("annotation-scale-up", index))
                                    .size(px(30.))
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(BORDER))
                                    .bg(rgb(0x11151a))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .child("+")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(annotation) = this.annotations.get_mut(index) {
                                            transform_annotation(annotation, 1.1, 0.0);
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .id(("annotation-rotate", index))
                                    .size(px(30.))
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(BORDER))
                                    .bg(rgb(0x11151a))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .child("↻")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(annotation) = this.annotations.get_mut(index) {
                                            transform_annotation(
                                                annotation,
                                                1.0,
                                                std::f32::consts::PI / 12.0,
                                            );
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(color_picker(
                                "selected-annotation-color",
                                app.object_color_picker_open,
                                annotation.color,
                                app.color_input.clone(),
                                picker_focus,
                                cx,
                                |this| {
                                    this.object_color_picker_open = !this.object_color_picker_open
                                },
                                move |this, color| {
                                    if let Some(annotation) = this.annotations.get_mut(index) {
                                        annotation.color = color;
                                    }
                                },
                                move |this, event| {
                                    if event.keystroke.key == "enter"
                                        && let Some(color) = parse_color(&this.color_input)
                                        && let Some(annotation) = this.annotations.get_mut(index)
                                    {
                                        annotation.color = color;
                                    }
                                },
                            )),
                    ),
            );
        }
    }

    if let Some((position, tool, size, color, generation)) = size_preview {
        let left = px((f32::from(position.x) - 310.0 - 36.0).max(0.0));
        let top = px((f32::from(position.y) - 48.0 - 21.0).max(0.0));
        let mut ghost = div().absolute().left(left).top(top).w(px(72.0)).h(px(42.0));
        if tool == Tool::Text {
            ghost = ghost
                .flex()
                .items_center()
                .justify_center()
                .text_size(px((size * 7.0).clamp(10.0, 72.0)))
                .text_color(gpui::rgba((color << 8) | 0x99))
                .child("Aa");
        } else {
            ghost = ghost.child(
                canvas(
                    move |_, _, _| {},
                    move |bounds, _, window, _| {
                        let x: f32 = bounds.origin.x.into();
                        let y: f32 = bounds.origin.y.into();
                        let w: f32 = bounds.size.width.into();
                        let h: f32 = bounds.size.height.into();
                        let start = point(px(x + 7.0), px(y + h * 0.65));
                        let end = point(px(x + w - 7.0), px(y + h * 0.35));
                        let mut path = PathBuilder::stroke(px(size));
                        path.move_to(start);
                        if tool == Tool::Pencil {
                            path.line_to(point(px(x + w * 0.35), px(y + h * 0.25)));
                            path.line_to(point(px(x + w * 0.62), px(y + h * 0.75)));
                        }
                        path.line_to(end);
                        if matches!(tool, Tool::Arrow | Tool::Angle) {
                            let dx = w - 14.0;
                            let dy = -h * 0.3;
                            let angle = dy.atan2(dx);
                            let head = (size * 6.0).min(18.0);
                            for wing in [
                                angle + std::f32::consts::FRAC_PI_6,
                                angle - std::f32::consts::FRAC_PI_6,
                            ] {
                                path.move_to(end);
                                path.line_to(point(
                                    px(x + w - 7.0 - head * wing.cos()),
                                    px(y + h * 0.35 - head * wing.sin()),
                                ));
                            }
                        }
                        if let Ok(path) = path.build() {
                            window.paint_path(path, gpui::rgba((color << 8) | 0x99));
                        }
                    },
                )
                .size_full(),
            );
        }
        viewport = viewport.child(ghost.with_animation(
            ("annotation-size-preview", generation),
            Animation::new(Duration::from_millis(500)).with_easing(ease_out_quint()),
            |element, progress| element.opacity(1.0 - progress),
        ));
    }

    viewport
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
                    app.annotations.len(),
                    if app.annotations.len() == 1 { "" } else { "s" }
                )),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_is_measured_counterclockwise_from_viewport_right() {
        let origin = point(px(10.0), px(10.0));
        assert_eq!(line_angle_degrees(origin, point(px(20.0), px(10.0))), 0);
        assert_eq!(line_angle_degrees(origin, point(px(10.0), px(0.0))), 90);
        assert_eq!(line_angle_degrees(origin, point(px(0.0), px(10.0))), 180);
        assert_eq!(line_angle_degrees(origin, point(px(10.0), px(20.0))), 270);
    }

    #[test]
    fn angle_endpoint_snaps_to_whole_degrees() {
        let origin = point(px(0.0), px(0.0));
        let endpoint = snap_to_angle_increment(
            origin,
            point(px(100.0), px(-40.0)),
            std::f32::consts::PI / 180.0,
        );
        assert_eq!(line_angle_degrees(origin, endpoint), 22);
    }
}
