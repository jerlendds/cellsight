use gpui::{Context, IntoElement, div, prelude::*, px, rgb};

const ACCENT: u32 = 0x5cc8ff;
const BORDER: u32 = 0x343c47;
const MUTED: u32 = 0x909aa7;
const SURFACE: u32 = 0x11151a;

/// A compact information trigger which reveals an inline explanatory channel.
///
/// The caller owns `open`, so channels can be restored or coordinated with
/// other application UI. `content` may be any GPUI element, including a tree
/// of interactive controls.
pub fn info_channel<T, E>(
    id: &'static str,
    label: &'static str,
    open: bool,
    content: E,
    cx: &mut Context<T>,
    toggle: impl Fn(&mut T) + 'static,
) -> impl IntoElement
where
    T: 'static,
    E: IntoElement,
{
    let trigger = div()
        .id(id)
        .flex()
        .items_center()
        .gap_2()
        .text_xs()
        .text_color(rgb(MUTED))
        .cursor_pointer()
        .hover(|style| style.text_color(rgb(ACCENT)))
        .on_click(cx.listener(move |this, _, _, cx| {
            toggle(this);
            cx.notify();
        }))
        .child(
            div()
                .size(px(16.))
                .rounded_full()
                .border_1()
                .border_color(rgb(if open { ACCENT } else { BORDER }))
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(if open { ACCENT } else { MUTED }))
                .child("i"),
        )
        .child(label)
        .child(if open { "⌃" } else { "⌄" });

    let mut root = div().flex().flex_col().gap_2().child(trigger);
    if open {
        root = root.child(
            div()
                .pl_3()
                .pr_3()
                .py_3()
                .border_l_2()
                .border_color(rgb(ACCENT))
                .rounded_r_md()
                .bg(rgb(SURFACE))
                .text_sm()
                .child(content),
        );
    }
    root
}
