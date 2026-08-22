use gpui::{
    Animation, AnimationExt, Context, IntoElement, Transformation, div, ease_out_quint, prelude::*,
    px, radians, rgb, svg,
};
use std::{f32::consts::PI, time::Duration};

const SURFACE: u32 = 0x171c22;
const TEXT: u32 = 0xe7ecf2;

pub fn section_header<T: 'static>(
    id: &'static str,
    title: &'static str,
    open: bool,
    cx: &mut Context<T>,
    toggle: impl Fn(&mut T) + 'static,
) -> impl IntoElement {
    let chevron_animation = format!("{id}-chevron-{}", if open { "opening" } else { "closing" });

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
        .child(
            svg()
                .data(include_bytes!("../../../src/assets/chevron-down.svg"))
                .size(px(16.))
                .text_color(rgb(TEXT))
                .with_animation(
                    chevron_animation,
                    Animation::new(Duration::from_millis(110)).with_easing(ease_out_quint()),
                    move |icon, progress| {
                        let rotation = if open {
                            progress * PI
                        } else {
                            (1.0 - progress) * PI
                        };
                        icon.with_transformation(Transformation::rotate(radians(rotation)))
                    },
                ),
        )
}
