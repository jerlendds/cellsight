use gpui::{Context, IntoElement, MouseButton, MouseDownEvent, div, prelude::*, px, rgb};

const ACCENT: u32 = 0x37b5e5;
const BORDER: u32 = 0x303942;
const TEXT: u32 = 0xd9e2ea;

pub fn slider<T: 'static>(
    id: &'static str,
    label: &'static str,
    value: u8,
    cx: &mut Context<T>,
    update: impl Fn(&mut T, u8) + 'static,
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
