use gpui::{
    Context, IntoElement, MouseButton, MouseDownEvent, SharedString, div, prelude::*, px, rgb,
};

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

/// A slider whose values are explicit named stops rather than a continuous range.
pub fn stepped_slider<T: 'static>(
    id: &'static str,
    label: &'static str,
    stops: Vec<SharedString>,
    selected: usize,
    cx: &mut Context<T>,
    update: impl Fn(&mut T, usize, &mut Context<T>) + Clone + 'static,
) -> impl IntoElement {
    let count = stops.len().max(1);
    let fraction = if count == 1 {
        0.0
    } else {
        selected as f32 / (count - 1) as f32
    };
    let mut track = div()
        .relative()
        .h(px(34.))
        .flex()
        .justify_between()
        .items_start()
        .child(
            div()
                .absolute()
                .top(px(6.))
                .left(px(6.))
                .right(px(6.))
                .h(px(2.))
                .bg(rgb(BORDER)),
        )
        .child(
            div()
                .absolute()
                .top(px(6.))
                .left(px(6.))
                .h(px(2.))
                .w(gpui::relative(fraction))
                .bg(rgb(ACCENT)),
        );
    for (index, stop) in stops.into_iter().enumerate() {
        let update = update.clone();
        track = track.child(
            div()
                .id((id, index))
                .relative()
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    update(this, index, cx);
                    cx.notify();
                }))
                .child(
                    div()
                        .size(px(13.))
                        .rounded_full()
                        .border_1()
                        .border_color(rgb(ACCENT))
                        .bg(rgb(if index == selected { ACCENT } else { BORDER })),
                )
                .child(div().text_xs().text_color(rgb(TEXT)).child(stop)),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().flex().justify_between().text_xs().child(label))
        .child(track)
}
