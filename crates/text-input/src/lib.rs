use gpui::{IntoElement, div, prelude::*, px, rgb};

const BORDER: u32 = 0x303942;
const MUTED: u32 = 0x89939e;

pub fn text_input(
    id: &'static str,
    label: &'static str,
    placeholder: &'static str,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(rgb(MUTED)).child(label))
        .child(
            div()
                .id(id)
                .h(px(36.))
                .px_3()
                .flex()
                .items_center()
                .rounded_md()
                .border_1()
                .border_color(rgb(BORDER))
                .bg(rgb(0x11151a))
                .text_sm()
                .text_color(rgb(MUTED))
                .child(placeholder),
        )
}
