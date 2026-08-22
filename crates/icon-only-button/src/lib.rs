use cellsight_theme::{ACCENT_BTN, ACTIVE_BTN, BORDER_BTN, SURFACE_BTN};
use gpui::{Context, IntoElement, div, prelude::*, px, rgb};

pub fn icon_only_button<T: 'static>(
    id: &'static str,
    icon: impl IntoElement,
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
            rgb(ACTIVE_BTN)
        } else {
            rgb(SURFACE_BTN)
        })
        .border_1()
        .border_color(if selected {
            rgb(ACCENT_BTN)
        } else {
            rgb(BORDER_BTN)
        })
        .child(icon)
        .hover(|s| s.border_color(rgb(BORDER_BTN)))
        .on_click(cx.listener(move |this, _, _, cx| {
            action(this);
            cx.notify();
        }))
}
