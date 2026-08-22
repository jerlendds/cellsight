use gpui::{
    Animation, AnimationExt, Context, IntoElement, SharedString, Transformation, deferred, div,
    ease_out_quint, prelude::*, px, radians, rgb, svg,
};
use std::{f32::consts::PI, time::Duration};

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
    let chevron_animation = format!(
        "{id}-chevron-{}",
        if is_open { "opening" } else { "closing" }
    );
    let trigger = div()
        .id(id)
        .flex()
        .min_w_0()
        .w_full()
        .gap_2()
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
        .child(div().min_w_0().flex_1().text_sm().truncate().child(value))
        .child(
            svg()
                .data(include_bytes!("../../../src/assets/chevron-down.svg"))
                .size(px(16.))
                .flex_none()
                .text_color(rgb(MUTED))
                .with_animation(
                    chevron_animation,
                    Animation::new(Duration::from_millis(180)).with_easing(ease_out_quint()),
                    move |icon, progress| {
                        let rotation = if is_open {
                            progress * PI
                        } else {
                            (1.0 - progress) * PI
                        };
                        icon.with_transformation(Transformation::rotate(radians(rotation)))
                    },
                ),
        );

    let mut field = div().relative().min_w_0().w_full().child(trigger);
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
                    .min_w_0()
                    .h(px(34.))
                    .px_3()
                    .flex()
                    .gap_2()
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
                    .child(div().min_w_0().flex_1().truncate().child(option))
                    .child(
                        div()
                            .flex_none()
                            .child(if index == selected { "✓" } else { "" }),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        select(this, index);
                        cx.notify();
                    })),
            );
        }
        field = field.child(deferred(menu.occlude()).with_priority(1));
    }

    div()
        .min_w_0()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(rgb(MUTED)).child(label))
        .child(field)
}
