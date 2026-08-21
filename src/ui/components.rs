use super::theme::{ACCENT, BORDER, MUTED, SURFACE, TEXT};
use crate::app::{CellSight, Dropdown};
use gpui::{
    Context, IntoElement, MouseButton, MouseDownEvent, SharedString, deferred, div, prelude::*, px,
    rgb,
};

pub(crate) fn section_header(
    id: &'static str,
    title: &'static str,
    open: bool,
    cx: &mut Context<CellSight>,
    toggle: impl Fn(&mut CellSight) + 'static,
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
pub(crate) fn selector(
    id: &'static str,
    label: &'static str,
    dropdown: Dropdown,
    is_open: bool,
    options: Vec<SharedString>,
    selected: usize,
    cx: &mut Context<CellSight>,
    select: impl Fn(&mut CellSight, usize) + Clone + 'static,
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
            this.open_dropdown = if this.open_dropdown == Some(dropdown) {
                None
            } else {
                Some(dropdown)
            };
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
                        this.open_dropdown = None;
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
pub(crate) fn slider(
    id: &'static str,
    label: &'static str,
    value: u8,
    cx: &mut Context<CellSight>,
    update: impl Fn(&mut CellSight, u8) + 'static,
) -> impl IntoElement {
    let fraction = value as f32 / 100.;
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .justify_between()
                .text_xs()
                .child(label)
                .child(format!("{value}%")),
        )
        .child(
            div()
                .id(id)
                .relative()
                .h(px(18.))
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                        let x: f32 = e.position.x.into();
                        update(this, ((x as i32).rem_euclid(240) * 100 / 240) as u8);
                        cx.notify();
                    }),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(7.))
                        .left_0()
                        .right_0()
                        .h(px(3.))
                        .rounded_full()
                        .bg(rgb(BORDER)),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(7.))
                        .left_0()
                        .h(px(3.))
                        .w(gpui::relative(fraction))
                        .rounded_full()
                        .bg(rgb(ACCENT)),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(2.))
                        .left(gpui::relative(fraction))
                        .size(px(13.))
                        .rounded_full()
                        .bg(rgb(TEXT)),
                ),
        )
}
pub(crate) fn button(
    id: &'static str,
    label: &'static str,
    primary: bool,
    active: bool,
    cx: &mut Context<CellSight>,
    action: impl Fn(&mut CellSight, &mut Context<CellSight>) + 'static,
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
pub(crate) fn icon_button(
    id: &'static str,
    icon: &'static str,
    label: &'static str,
    active: bool,
    cx: &mut Context<CellSight>,
    action: impl Fn(&mut CellSight) + 'static,
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
pub(crate) fn icon_only_button(
    id: &'static str,
    icon: &'static str,
    selected: bool,
    cx: &mut Context<CellSight>,
    action: impl Fn(&mut CellSight) + 'static,
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
pub(crate) fn text_input(
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
