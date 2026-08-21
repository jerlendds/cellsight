use gpui::{Context, IntoElement, div, prelude::*, px, rgb};

const ACCENT: u32 = 0x37b5e5;
const BORDER: u32 = 0x303942;
const SURFACE: u32 = 0x171c22;

pub fn button<T: 'static>(
    id: &'static str,
    label: &'static str,
    primary: bool,
    active: bool,
    cx: &mut Context<T>,
    action: impl Fn(&mut T, &mut Context<T>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(36.))
        .px_3()
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .border_1()
        .border_color(if primary || active {
            rgb(ACCENT)
        } else {
            rgb(BORDER)
        })
        .bg(if primary {
            rgb(0x174b63)
        } else if active {
            rgb(0x42252a)
        } else {
            rgb(SURFACE)
        })
        .hover(|s| s.opacity(0.85))
        .active(|s| s.opacity(0.7))
        .text_sm()
        .child(label)
        .on_click(cx.listener(move |this, _, _, cx| {
            action(this, cx);
            cx.notify();
        }))
}
