use crate::app::{CellSight, Dropdown};
use cellsight_button::button;
use cellsight_focus_profile::{FocusResolution, FocusView};
use cellsight_info_channel::info_channel;
use cellsight_section_header::section_header;
use cellsight_selector::selector;
use cellsight_slider::{slider, stepped_slider};
use cellsight_text_input::text_input;
use cellsight_theme::{BORDER, PANEL};
use gpui::{
    Animation, AnimationExt, Context, IntoElement, div, ease_out_quint, prelude::*, px, rgb,
};
use std::time::Duration;

fn focus_view_index(view: FocusView) -> usize {
    match view {
        FocusView::Original => 0,
        FocusView::Depth => 1,
        FocusView::Falloff => 2,
        FocusView::Confidence => 3,
        FocusView::AllInFocus => 4,
    }
}

fn focus_view_at(index: usize) -> FocusView {
    [
        FocusView::Original,
        FocusView::Depth,
        FocusView::Falloff,
        FocusView::Confidence,
        FocusView::AllInFocus,
    ][index]
}

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
                    app.open_dropdown == Some(Dropdown::Camera),
                    app.cameras.clone(),
                    app.camera,
                    cx,
                    |s| {
                        s.open_dropdown = if s.open_dropdown == Some(Dropdown::Camera) {
                            None
                        } else {
                            Some(Dropdown::Camera)
                        }
                    },
                    |s, index| {
                        s.camera = index;
                        s.open_dropdown = None;
                    },
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
                    Animation::new(Duration::from_millis(110)).with_easing(ease_out_quint()),
                    |e, d| e.max_h(px(300. * d)).overflow_hidden().opacity(d),
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
                    app.open_dropdown == Some(Dropdown::Format),
                    app.formats.clone(),
                    app.format,
                    cx,
                    |s| {
                        s.open_dropdown = if s.open_dropdown == Some(Dropdown::Format) {
                            None
                        } else {
                            Some(Dropdown::Format)
                        }
                    },
                    |s, index| {
                        s.format = index;
                        s.open_dropdown = None;
                    },
                ))
                .child(selector(
                    "resolution-select",
                    "Resolution",
                    app.open_dropdown == Some(Dropdown::Resolution),
                    app.resolutions.clone(),
                    app.resolution,
                    cx,
                    |s| {
                        s.open_dropdown = if s.open_dropdown == Some(Dropdown::Resolution) {
                            None
                        } else {
                            Some(Dropdown::Resolution)
                        }
                    },
                    |s, index| {
                        s.resolution = index;
                        s.open_dropdown = None;
                    },
                ))
                .child(selector(
                    "fps-select",
                    "Frame rate",
                    app.open_dropdown == Some(Dropdown::FrameRate),
                    app.fps_values
                        .iter()
                        .map(|fps| format!("{fps} fps").into())
                        .collect(),
                    app.fps,
                    cx,
                    |s| {
                        s.open_dropdown = if s.open_dropdown == Some(Dropdown::FrameRate) {
                            None
                        } else {
                            Some(Dropdown::FrameRate)
                        }
                    },
                    |s, index| {
                        s.fps = index;
                        s.open_dropdown = None;
                    },
                ))
                .child(button(
                    "focus-sweep",
                    if app.recording { "Finish focus sweep" } else { "Start focus sweep" },
                    !app.recording,
                    app.recording,
                    cx,
                    |s, cx| s.toggle_focus_sweep(cx),
                ))
                .child(button(
                    "comparison-toggle",
                    if app.comparison_enabled { "Hide comparison" } else { "Compare images" },
                    app.comparison_enabled,
                    false,
                    cx,
                    |s, _| s.comparison_enabled = !s.comparison_enabled,
                ))
                .child(selector(
                    "compare-left-select",
                    "Comparison left",
                    app.open_dropdown == Some(Dropdown::CompareLeft),
                    vec!["Original".into(), "Depth".into(), "Falloff".into(), "Confidence".into(), "All in focus".into()],
                    focus_view_index(app.comparison_left),
                    cx,
                    |s| {
                        s.open_dropdown = if s.open_dropdown == Some(Dropdown::CompareLeft) { None } else { Some(Dropdown::CompareLeft) }
                    },
                    |s, index| {
                        s.set_comparison_left(focus_view_at(index));
                        s.open_dropdown = None;
                    },
                ))
                .child(selector(
                    "focus-view-select",
                    "Comparison right",
                    app.open_dropdown == Some(Dropdown::CompareRight),
                    vec![
                        "Original".into(),
                        "Depth".into(),
                        "Falloff".into(),
                        "Confidence".into(),
                        "All in focus".into(),
                    ],
                    focus_view_index(app.comparison_right),
                    cx,
                    |s| {
                        s.open_dropdown = if s.open_dropdown == Some(Dropdown::CompareRight) {
                            None
                        } else {
                            Some(Dropdown::CompareRight)
                        }
                    },
                    |s, index| {
                        s.set_comparison_right(focus_view_at(index));
                        s.open_dropdown = None;
                    },
                ))
                .child(stepped_slider(
                    "focus-resolution",
                    "Depth resolution",
                    vec!["320".into(), "640".into(), "960".into(), "Full".into()],
                    match app.focus_resolution {
                        FocusResolution::Width320 => 0,
                        FocusResolution::Width640 => 1,
                        FocusResolution::Width960 => 2,
                        FocusResolution::Full => 3,
                    },
                    cx,
                    |s, index, cx| {
                        let resolution = [
                            FocusResolution::Width320,
                            FocusResolution::Width640,
                            FocusResolution::Width960,
                            FocusResolution::Full,
                        ][index];
                        s.set_focus_resolution(resolution, cx);
                    },
                ))
                .child(button(
                    "focus-region",
                    if app.focus_region.is_some() { "Change analysis region" } else { "Select analysis region" },
                    app.selecting_focus_region,
                    false,
                    cx,
                    |s, _| {
                        if s.recording { return; }
                        s.selecting_focus_region = true;
                        s.focus_region_anchor = None;
                    },
                ))
                .child(button(
                    "focus-region-clear",
                    "Use full frame",
                    false,
                    app.focus_region.is_none(),
                    cx,
                    |s, cx| {
                        if s.recording { return; }
                        s.focus_region = None;
                        s.focus_region_anchor = None;
                        s.selecting_focus_region = false;
                        s.rerun_captured_sweep(cx);
                    },
                ))
                .child(info_channel(
                    "focus-depth-info",
                    "How focus depth works",
                    app.focus_info_open,
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child("Sweep the focus steadily in one direction through the specimen. Each region becomes sharpest at a different frame, and that frame order becomes its relative depth.")
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x909aa7))
                                .child("Blue reaches focus near the start of the sweep; red reaches focus near the end. Reversing the sweep reverses the depth map."),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x909aa7))
                                .child("Falloff shows narrow focus peaks, Confidence highlights reliable textured regions, and All in focus composites every region at its sharpest plane."),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x909aa7))
                                .child(format!(
                                    "Captured sweeps are stored in {}{}",
                                    app.app_data_dir.display(),
                                    app.captured_sweep
                                        .as_ref()
                                        .map(|capture| format!(
                                            " · latest: {}",
                                            capture.directory.display()
                                        ))
                                        .unwrap_or_default()
                                )),
                        ),
                    cx,
                    |s| s.focus_info_open = !s.focus_info_open,
                ))
                .with_animation(
                    "capture-reveal",
                    Animation::new(Duration::from_millis(110)).with_easing(ease_out_quint()),
                    |e, d| e.max_h(px(1200. * d)).overflow_hidden().opacity(d),
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
                    Animation::new(Duration::from_millis(110)).with_easing(ease_out_quint()),
                    |e, d| e.max_h(px(500. * d)).overflow_hidden().opacity(d),
                ),
        );
    }
    side
}
