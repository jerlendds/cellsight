use cellsight_theme::{ACCENT, ACTIVE_BTN, BORDER_BTN, SURFACE_BTN, TEXT};
use gpui::{
    Animation, AnimationExt, Context, FocusHandle, IntoElement, KeyDownEvent, MouseButton,
    SharedString, deferred, div, ease_out_quint, prelude::*, px, rgb, svg,
};
use std::time::Duration;

const PANEL_HEIGHT: f32 = 224.;

pub const HIGH_CONTRAST_COLORS: [u32; 11] = [
    0xff0000, 0x00ff00, 0x0000ff, 0x000000, 0xffffff, 0x00ffff, 0xffe066, 0xff5cc8, 0xff8a4c,
    0xb57cff, 0xffb800,
];

pub fn parse_color(value: &str) -> Option<u32> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        if hex.len() == 6 {
            return u32::from_str_radix(hex, 16).ok();
        }
    }

    let inner = value
        .strip_prefix("rgb(")
        .or_else(|| value.strip_prefix("RGB("))?
        .strip_suffix(')')?;
    let mut channels = inner.split(',').map(str::trim);
    let red = channels.next()?.parse::<u8>().ok()?;
    let green = channels.next()?.parse::<u8>().ok()?;
    let blue = channels.next()?.parse::<u8>().ok()?;
    if channels.next().is_some() {
        return None;
    }
    Some(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
}

pub fn color_picker<T: 'static>(
    id: &'static str,
    is_open: bool,
    selected: u32,
    input: SharedString,
    input_focus: FocusHandle,
    cx: &mut Context<T>,
    toggle: impl Fn(&mut T) + 'static,
    select: impl Fn(&mut T, u32) + Clone + 'static,
    edit: impl Fn(&mut T, &KeyDownEvent) + 'static,
) -> impl IntoElement {
    let animation_id = format!("{id}-{}", if is_open { "opening" } else { "closing" });
    let panel_animation_id = format!("{animation_id}-panel");
    let palette_icon: &'static [u8] = if is_open {
        &include_bytes!("../../../src/assets/palette-off.svg")[..]
    } else {
        &include_bytes!("../../../src/assets/palette.svg")[..]
    };

    let button = div()
        .id((id, 100usize))
        .size(px(34.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .bg(if is_open {
            rgb(ACTIVE_BTN)
        } else {
            rgb(SURFACE_BTN)
        })
        .border_1()
        .border_color(if is_open {
            rgb(ACCENT)
        } else {
            rgb(BORDER_BTN)
        })
        .hover(|style| style.border_color(rgb(ACCENT)))
        .child(svg().data(palette_icon).size(px(18.)).text_color(rgb(TEXT)))
        .on_click(cx.listener(move |this, _, _, cx| {
            toggle(this);
            cx.notify();
        }));

    let input_display = if input.is_empty() {
        "#RRGGBB or rgb(…)".into()
    } else {
        input
    };
    let focus_on_click = input_focus.clone();
    let input = div()
        .id((id, 101usize))
        .track_focus(&input_focus)
        .h(px(34.))
        .flex_none()
        .px_2()
        .flex()
        .items_center()
        .rounded_md()
        .border_1()
        .border_color(rgb(BORDER_BTN))
        .bg(rgb(SURFACE_BTN))
        .text_sm()
        .text_color(rgb(TEXT))
        .cursor_text()
        .child(div().min_w_0().flex_1().truncate().child(input_display))
        .child(
            div()
                .size(px(22.0))
                .flex_none()
                .rounded_sm()
                .border_1()
                .border_color(rgb(BORDER_BTN))
                .bg(rgb(selected)),
        )
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            focus_on_click.focus(window, cx);
        })
        .on_key_down(cx.listener(move |this, event, _, cx| {
            edit(this, event);
            cx.notify();
        }));

    let mut colors = div()
        .id((id, 102usize))
        .bg(rgb(SURFACE_BTN))
        .border_l_1()
        .border_r_1()
        .border_b_1()
        .border_color(rgb(ACTIVE_BTN))
        .absolute()
        .top(px(40.))
        .right_0()
        .w(px(34.))
        .flex()
        .flex_col()
        .gap_2()
        .max_h(px(182.))
        .overflow_y_scroll();
    for (index, color) in HIGH_CONTRAST_COLORS.into_iter().enumerate() {
        let select = select.clone();
        colors = colors.child(
            div()
                .id((id, index))
                .size(px(32.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .border_2()
                .border_color(if color == selected {
                    rgb(ACCENT)
                } else {
                    rgb(BORDER_BTN)
                })
                .hover(|style| style.bg(rgb(ACTIVE_BTN)))
                .child(div().size(px(22.)).rounded_sm().bg(rgb(color)))
                .on_click(cx.listener(move |this, _, _, cx| {
                    select(this, color);
                    cx.notify();
                })),
        );
    }

    let panel = div()
        .absolute()
        .top_0()
        .right_0()
        .w(px(190.))
        .h(px(PANEL_HEIGHT))
        .relative()
        .flex()
        .flex_col()
        .child(input)
        .child(colors)
        .with_animation(
            panel_animation_id,
            Animation::new(Duration::from_millis(180)).with_easing(ease_out_quint()),
            move |panel, progress| {
                let reveal = if is_open { progress } else { 1. - progress };
                panel.top(px(-PANEL_HEIGHT * (1. - reveal)))
            },
        );

    let reveal = div()
        .absolute()
        .top(px(38.))
        .right_0()
        .w(px(190.))
        .overflow_hidden()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .child(panel)
        .occlude()
        .with_animation(
            animation_id,
            Animation::new(Duration::from_millis(180)).with_easing(ease_out_quint()),
            move |element, progress| {
                let reveal = if is_open { progress } else { 1. - progress };
                element.h(px(PANEL_HEIGHT * reveal)).opacity(reveal)
            },
        );

    div()
        .relative()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .child(button)
        .when(is_open, |element| {
            element.child(deferred(reveal).with_priority(2))
        })
}

#[cfg(test)]
mod tests {
    use super::parse_color;

    #[test]
    fn parses_hex_and_rgb_colors() {
        assert_eq!(parse_color("#5CC8FF"), Some(0x5cc8ff));
        assert_eq!(parse_color("rgb(255, 92, 200)"), Some(0xff5cc8));
        assert_eq!(parse_color("rgb(256, 0, 0)"), None);
        assert_eq!(parse_color("nope"), None);
    }
}
