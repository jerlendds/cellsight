use super::{
    components::{button, section_header, selector, slider, text_input},
    theme::{BORDER, PANEL},
};
use crate::app::{CellSight, Dropdown};
use gpui::{
    Animation, AnimationExt, Context, IntoElement, div, ease_out_quint, prelude::*, px, rgb,
};
use std::time::Duration;

pub(crate) fn render(app: &CellSight, cx: &mut Context<CellSight>) -> impl IntoElement + use<> {
    let mut side = div()
        .id("side-panel")
        .w(px(310.))
        .h_full()
        .flex_none()
        .bg(rgb(PANEL))
        .border_r_1()
        .border_color(rgb(BORDER))
        .overflow_y_scroll();
    side = side.child(section_header(
        "camera-section",
        "CAMERA",
        app.camera_open,
        cx,
        |s| s.camera_open = !s.camera_open,
    ));
    if app.camera_open {
        side = side.child(
            div()
                .px_4()
                .pb_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(selector(
                    "camera-select",
                    "Device",
                    Dropdown::Camera,
                    app.open_dropdown == Some(Dropdown::Camera),
                    app.cameras.clone(),
                    app.camera,
                    cx,
                    |s, index| s.camera = index,
                ))
                .child(button(
                    "stream-toggle",
                    if app.streaming {
                        "Disconnect"
                    } else {
                        "Connect camera"
                    },
                    !app.streaming,
                    false,
                    cx,
                    |s, cx| {
                        if s.streaming {
                            s.stop_camera()
                        } else {
                            s.start_camera(cx)
                        }
                    },
                ))
                .with_animation(
                    "camera-reveal",
                    Animation::new(Duration::from_millis(180)).with_easing(ease_out_quint()),
                    |e, d| e.opacity(d),
                ),
        );
    }
    side = side
        .child(div().h(px(1.)).bg(rgb(BORDER)))
        .child(section_header(
            "capture-section",
            "CAPTURE",
            app.capture_open,
            cx,
            |s| s.capture_open = !s.capture_open,
        ));
    if app.capture_open {
        side = side.child(
            div()
                .px_4()
                .pb_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(selector(
                    "format-select",
                    "Format",
                    Dropdown::Format,
                    app.open_dropdown == Some(Dropdown::Format),
                    app.formats.clone(),
                    app.format,
                    cx,
                    |s, index| s.format = index,
                ))
                .child(selector(
                    "resolution-select",
                    "Resolution",
                    Dropdown::Resolution,
                    app.open_dropdown == Some(Dropdown::Resolution),
                    app.resolutions.clone(),
                    app.resolution,
                    cx,
                    |s, index| s.resolution = index,
                ))
                .child(selector(
                    "fps-select",
                    "Frame rate",
                    Dropdown::FrameRate,
                    app.open_dropdown == Some(Dropdown::FrameRate),
                    app.fps_values
                        .iter()
                        .map(|fps| format!("{fps} fps").into())
                        .collect(),
                    app.fps,
                    cx,
                    |s, index| s.fps = index,
                ))
                .with_animation(
                    "capture-reveal",
                    Animation::new(Duration::from_millis(180)).with_easing(ease_out_quint()),
                    |e, d| e.opacity(d),
                ),
        );
    }
    side = side
        .child(div().h(px(1.)).bg(rgb(BORDER)))
        .child(section_header(
            "controls-section",
            "IMAGE CONTROLS",
            app.controls_open,
            cx,
            |s| s.controls_open = !s.controls_open,
        ));
    if app.controls_open {
        side = side.child(
            div()
                .px_4()
                .pb_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(slider("exposure", "Exposure", app.exposure, cx, |s, v| {
                    s.exposure = v
                }))
                .child(slider("gain", "Gain", app.gain, cx, |s, v| s.gain = v))
                .child(text_input(
                    "annotation-label",
                    "Annotation label",
                    "Sample region…",
                ))
                .with_animation(
                    "controls-reveal",
                    Animation::new(Duration::from_millis(180)).with_easing(ease_out_quint()),
                    |e, d| e.opacity(d),
                ),
        );
    }
    side
}
