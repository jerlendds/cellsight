use gpui::{Context, IntoElement, SharedString, deferred, div, prelude::*, px, rgb};

const ACCENT: u32 = 0x37b5e5;
const BORDER: u32 = 0x303942;
const MUTED: u32 = 0x89939e;
const SURFACE: u32 = 0x171c22;

pub fn selector<T: 'static>(
    id: &'static str,
    label: &'static str,
    is_open: bool,
    options: Vec<SharedString>,
    selected: usize,
    cx: &mut Context<T>,
    toggle: impl Fn(&mut T) + 'static,
    select: impl Fn(&mut T, usize) + Clone + 'static,
) -> impl IntoElement {
    let value = options[selected].clone();
    let trigger = div()
        .id(id)
        .flex()
        .items_center()
        .justify_between()
        .h(px(36.))
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgb(0x11151a))
        .cursor_pointer()
        .hover(|s| s.border_color(rgb(ACCENT)))
        .on_click(cx.listener(move |this, _, _, cx| {
            toggle(this);
            cx.notify();
        }))
        .child(div().text_sm().child(value))
        .child(
            div()
                .text_color(rgb(MUTED))
                .child(if is_open { "⌃" } else { "⌄" }),
        );

    let mut field = div().relative().child(trigger);
    if is_open {
        let mut menu = div()
            .absolute()
            .top_full()
            .left_0()
            .right_0()
            .mt_1()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(rgb(BORDER))
            .bg(rgb(0x11151a));
        for (index, option) in options.into_iter().enumerate() {
            let select = select.clone();
            menu = menu.child(
                div()
                    .id((id, index))
                    .h(px(34.))
                    .px_3()
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .text_sm()
                    .bg(if index == selected {
                        rgb(SURFACE)
                    } else {
                        rgb(0x11151a)
                    })
                    .hover(|s| s.bg(rgb(SURFACE)))
                    .child(option)
                    .child(if index == selected { "✓" } else { "" })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        select(this, index);
                        cx.notify();
                    })),
            );
        }
        field = field.child(deferred(menu.occlude()).with_priority(1));
    }

    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(rgb(MUTED)).child(label))
        .child(field)
}
