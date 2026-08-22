use crate::app::{CellSight, Tool};
use cellsight_color_picker::{color_picker, parse_color};
use cellsight_icon_only_button::icon_only_button;
use cellsight_theme::{
    ACCENT, ACCENT_BTN, ACTIVE_BTN, BORDER, BORDER_BTN, MUTED, PANEL, SURFACE_BTN, TEXT,
};
use gpui::{
    Animation, AnimationExt, Context, IntoElement, div, ease_out_quint, prelude::*, px, rgb, svg,
};
use std::time::Duration;

pub(crate) fn render(app: &mut CellSight, cx: &mut Context<CellSight>) -> impl IntoElement + use<> {
    let annotations_open = app.annotations_open;
    let color_input_focus = app
        .color_input_focus
        .get_or_insert_with(|| cx.focus_handle())
        .clone();
    app.annotation_text_focus
        .get_or_insert_with(|| cx.focus_handle());
    let annotation_animation = if annotations_open {
        "annotations-opening"
    } else {
        "annotations-closing"
    };

    div()
        .relative()
        .h(px(48.))
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .bg(rgb(PANEL))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .size(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            svg()
                                .data(include_bytes!("../assets/microscope.svg"))
                                .size(px(24.))
                                .text_color(rgb(TEXT)),
                        ),
                )
                .child(
                    div()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child("cellsight"),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .child(
                    div()
                        .h(px(34.))
                        .overflow_hidden()
                        .child(
                            div()
                                .w(px(244.))
                                .flex_none()
                                .flex()
                                .gap_2()
                                .child(icon_only_button(
                                    "selection-tool",
                                    svg()
                                        .data(include_bytes!("../assets/hand.svg"))
                                        .size(px(18.))
                                        .text_color(rgb(TEXT)),
                                    app.tool == Tool::Select,
                                    cx,
                                    |s| {
                                        s.tool = Tool::Select;
                                        s.editing_annotation = None;
                                    },
                                ))
                                .child(icon_only_button(
                                    "line-tool",
                                    svg()
                                        .data(include_bytes!("../assets/line.svg"))
                                        .size(px(18.))
                                        .text_color(rgb(TEXT)),
                                    app.tool == Tool::Line,
                                    cx,
                                    |s| s.tool = Tool::Line,
                                ))
                                .child(icon_only_button(
                                    "angle-tool",
                                    svg()
                                        .data(include_bytes!("../assets/angle.svg"))
                                        .size(px(18.))
                                        .text_color(rgb(TEXT)),
                                    app.tool == Tool::Angle,
                                    cx,
                                    |s| s.tool = Tool::Angle,
                                ))
                                .child(icon_only_button(
                                    "arrow-tool",
                                    svg()
                                        .data(include_bytes!("../assets/arrow-right-circle.svg"))
                                        .size(px(18.))
                                        .text_color(rgb(TEXT)),
                                    app.tool == Tool::Arrow,
                                    cx,
                                    |s| s.tool = Tool::Arrow,
                                ))
                                .child(icon_only_button(
                                    "pencil-tool",
                                    svg()
                                        .data(include_bytes!("../assets/sketching.svg"))
                                        .size(px(18.))
                                        .text_color(rgb(TEXT)),
                                    app.tool == Tool::Pencil,
                                    cx,
                                    |s| s.tool = Tool::Pencil,
                                ))
                                .child(icon_only_button(
                                    "text-tool",
                                    svg()
                                        .data(include_bytes!("../assets/text-size.svg"))
                                        .size(px(18.))
                                        .text_color(rgb(TEXT)),
                                    app.tool == Tool::Text,
                                    cx,
                                    |s| s.tool = Tool::Text,
                                )),
                        )
                        .with_animation(
                            annotation_animation,
                            Animation::new(Duration::from_millis(180))
                                .with_easing(ease_out_quint()),
                            move |element, progress| {
                                let reveal = if annotations_open {
                                    progress
                                } else {
                                    1.0 - progress
                                };
                                element.w(px(244. * reveal)).opacity(reveal)
                            },
                        ),
                )
                .child(
                    div()
                        .h(px(34.))
                        .ml(px(8.))
                        .when(annotations_open, |element| {
                            element.child(color_picker(
                                "annotation-color",
                                app.color_picker_open,
                                app.annotation_color,
                                app.color_input.clone(),
                                color_input_focus,
                                cx,
                                |s| s.color_picker_open = !s.color_picker_open,
                                |s, color| {
                                    s.annotation_color = color;
                                    s.color_input = format!("#{color:06X}").into();
                                },
                                |s, event| {
                                    if event.keystroke.key == "backspace" {
                                        let mut value = s.color_input.to_string();
                                        value.pop();
                                        s.color_input = value.into();
                                    } else if event.keystroke.key == "enter" {
                                        if let Some(color) = parse_color(&s.color_input) {
                                            s.annotation_color = color;
                                            s.color_input = format!("#{color:06X}").into();
                                        }
                                    } else if let Some(characters) = &event.keystroke.key_char {
                                        let accepted = characters.chars().all(|character| {
                                            character.is_ascii_hexdigit()
                                                || matches!(character, '#' | '(' | ')' | ',' | ' ')
                                                || matches!(
                                                    character,
                                                    'r' | 'g' | 'b' | 'R' | 'G' | 'B'
                                                )
                                        });
                                        if accepted && s.color_input.len() + characters.len() <= 24
                                        {
                                            let mut value = s.color_input.to_string();
                                            value.push_str(characters);
                                            s.color_input = value.into();
                                        }
                                    }
                                },
                            ))
                        })
                        .with_animation(
                            if annotations_open {
                                "annotation-color-opening"
                            } else {
                                "annotation-color-closing"
                            },
                            Animation::new(Duration::from_millis(180))
                                .with_easing(ease_out_quint()),
                            move |element, progress| {
                                let reveal = if annotations_open {
                                    progress
                                } else {
                                    1.0 - progress
                                };
                                element.w(px(34. * reveal)).opacity(reveal)
                            },
                        ),
                )
                .child(
                    div()
                        .id("annotation-toggle")
                        .ml(px(8.))
                        .size(px(34.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .cursor_pointer()
                        .text_color(rgb(TEXT))
                        .bg(if annotations_open {
                            rgb(ACTIVE_BTN)
                        } else {
                            rgb(SURFACE_BTN)
                        })
                        .border_1()
                        .border_color(if annotations_open {
                            rgb(ACCENT_BTN)
                        } else {
                            rgb(BORDER_BTN)
                        })
                        .hover(|s| s.border_color(rgb(ACCENT)))
                        .child(if annotations_open {
                            svg()
                                .data(include_bytes!("../assets/x.svg"))
                                .size(px(18.))
                                .text_color(rgb(TEXT))
                        } else {
                            svg()
                                .data(include_bytes!("../assets/sparkle-highlight.svg"))
                                .size(px(18.))
                                .text_color(rgb(TEXT))
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.annotations_open = !this.annotations_open;
                            if !this.annotations_open {
                                this.color_picker_open = false;
                                this.editing_annotation = None;
                            }
                            this.drawing = false;
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .absolute()
                .left(px(130.))
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(div().size(px(7.)).rounded_full().bg(if app.streaming {
                            rgb(0x55d68b)
                        } else {
                            rgb(0x68717c)
                        }))
                        .child(if app.streaming { "LIVE" } else { "OFFLINE" }),
                ),
        )
}
