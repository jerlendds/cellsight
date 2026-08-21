use gpui::{Context, IntoElement, div, prelude::*, px, rgb};

const ACCENT: u32 = 0x37b5e5;
const BORDER: u32 = 0x303942;
const SURFACE: u32 = 0x171c22;

pub fn icon_button<T: 'static>(
    id: &'static str,
    icon: &'static str,
    label: &'static str,
    active: bool,
    cx: &mut Context<T>,
    action: impl Fn(&mut T) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(36.))
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .rounded_md()
        .cursor_pointer()
        .border_1()
        .border_color(if active { rgb(ACCENT) } else { rgb(BORDER) })
        .bg(if active { rgb(0x42252a) } else { rgb(SURFACE) })
        .hover(|s| s.border_color(rgb(ACCENT)))
        .child(icon)
        .child(div().text_sm().child(label))
        .on_click(cx.listener(move |this, _, _, cx| {
            action(this);
            cx.notify();
        }))
}
