use gpui::{Context, IntoElement, div, prelude::*, px, rgb};

const ACCENT: u32 = 0x37b5e5;
const BORDER: u32 = 0x303942;
const SURFACE: u32 = 0x171c22;

pub fn icon_only_button<T: 'static>(
    id: &'static str,
    icon: &'static str,
    selected: bool,
    cx: &mut Context<T>,
    action: impl Fn(&mut T) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(34.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .bg(if selected {
            rgb(0x174b63)
        } else {
            rgb(SURFACE)
        })
        .border_1()
        .border_color(if selected { rgb(ACCENT) } else { rgb(BORDER) })
        .child(icon)
        .hover(|s| s.border_color(rgb(ACCENT)))
        .on_click(cx.listener(move |this, _, _, cx| {
            action(this);
            cx.notify();
        }))
}
