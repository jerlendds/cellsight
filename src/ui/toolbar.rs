use super::theme::{ACCENT, BORDER, MUTED, PANEL};
use crate::app::{CellSight, Tool};
use cellsight_icon_button::icon_button;
use cellsight_icon_only_button::icon_only_button;
use gpui::{Context, IntoElement, div, prelude::*, px, rgb};

pub(crate) fn render(app: &CellSight, cx: &mut Context<CellSight>) -> impl IntoElement + use<> {
    div()
        .h(px(54.))
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
                .gap_3()
                .child(
                    div()
                        .size(px(28.))
                        .rounded_md()
                        .bg(rgb(ACCENT))
                        .text_color(rgb(0x071117))
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("C"),
                )
                .child(
                    div()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("CellSight"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child("CAMERA WORKBENCH"),
                ),
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(icon_only_button(
                    "pointer-tool",
                    "↖",
                    app.tool == Tool::Pointer,
                    cx,
                    |s| s.tool = Tool::Pointer,
                ))
                .child(icon_only_button(
                    "pen-tool",
                    "✎",
                    app.tool == Tool::Pen,
                    cx,
                    |s| s.tool = Tool::Pen,
                ))
                .child(icon_only_button(
                    "ruler-tool",
                    "⌁",
                    app.tool == Tool::Ruler,
                    cx,
                    |s| s.tool = Tool::Ruler,
                ))
                .child(icon_only_button(
                    "clear-annotations",
                    "⌫",
                    false,
                    cx,
                    |s| s.strokes.clear(),
                )),
        )
        .child(
            div()
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
                )
                .child(icon_button(
                    "record",
                    if app.recording { "■" } else { "●" },
                    if app.recording { "Stop" } else { "Record" },
                    app.recording,
                    cx,
                    |s| s.recording = !s.recording,
                )),
        )
}
