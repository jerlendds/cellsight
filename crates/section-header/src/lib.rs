use gpui::{Context, IntoElement, div, prelude::*, rgb};

const SURFACE: u32 = 0x171c22;

pub fn section_header<T: 'static>(
    id: &'static str,
    title: &'static str,
    open: bool,
    cx: &mut Context<T>,
    toggle: impl Fn(&mut T) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .py_3()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(SURFACE)))
        .on_click(cx.listener(move |this, _, _, cx| {
            toggle(this);
            cx.notify();
        }))
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(if open { "⌃" } else { "⌄" })
}
