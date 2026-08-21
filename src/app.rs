use cellsight_focus_profile::{FocusResolution, FocusResult, FocusSweep, FocusView};
use gpui::{Pixels, RenderImage, SharedString};
use std::{
    path::PathBuf,
    process::Child,
    sync::{Arc, Mutex, atomic::AtomicBool},
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tool {
    Pointer,
    Pen,
    Ruler,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Dropdown {
    Camera,
    Format,
    Resolution,
    FrameRate,
    CompareLeft,
    CompareRight,
}

pub(crate) struct CellSight {
    pub(crate) camera_open: bool,
    pub(crate) capture_open: bool,
    pub(crate) controls_open: bool,
    pub(crate) focus_info_open: bool,
    pub(crate) open_dropdown: Option<Dropdown>,
    pub(crate) cameras: Vec<SharedString>,
    pub(crate) camera: usize,
    pub(crate) formats: Vec<SharedString>,
    pub(crate) format: usize,
    pub(crate) resolutions: Vec<SharedString>,
    pub(crate) resolution: usize,
    pub(crate) fps_values: Vec<u16>,
    pub(crate) fps: usize,
    pub(crate) exposure: u8,
    pub(crate) gain: u8,
    pub(crate) streaming: bool,
    pub(crate) camera_frame: Option<Arc<RenderImage>>,
    // Keep the two most recently painted frames alive. GPUI's scene can still
    // reference the previous frame while the next one is being submitted.
    pub(crate) current_rendered_frame: Option<Arc<RenderImage>>,
    pub(crate) previous_rendered_frame: Option<Arc<RenderImage>>,
    pub(crate) camera_error: Option<SharedString>,
    pub(crate) capture_cancel: Option<Arc<AtomicBool>>,
    pub(crate) capture_process: Option<Arc<Mutex<Child>>>,
    pub(crate) recording: bool,
    pub(crate) app_data_dir: PathBuf,
    pub(crate) sweep_recorder: Option<crate::camera::SweepRecorder>,
    pub(crate) captured_sweep: Option<crate::camera::CapturedSweep>,
    pub(crate) focus_processing: bool,
    pub(crate) focus_progress: u8,
    pub(crate) focus_analysis_cancel: Option<Arc<AtomicBool>>,
    pub(crate) focus_sweep: FocusSweep,
    pub(crate) focus_result: Option<FocusResult>,
    pub(crate) focus_frame: Option<Arc<RenderImage>>,
    pub(crate) comparison_left_frame: Option<Arc<RenderImage>>,
    pub(crate) comparison_right_frame: Option<Arc<RenderImage>>,
    pub(crate) focus_view: FocusView,
    pub(crate) comparison_enabled: bool,
    pub(crate) comparison_left: FocusView,
    pub(crate) comparison_right: FocusView,
    pub(crate) comparison_split: f32,
    pub(crate) dragging_comparison: bool,
    pub(crate) focus_resolution: FocusResolution,
    pub(crate) focus_region: Option<[f32; 4]>,
    pub(crate) focus_region_anchor: Option<[f32; 2]>,
    pub(crate) selecting_focus_region: bool,
    pub(crate) tool: Tool,
    pub(crate) drawing: bool,
    pub(crate) strokes: Vec<Vec<gpui::Point<Pixels>>>,
}

impl CellSight {
    pub(crate) fn new() -> Self {
        Self {
            camera_open: true,
            capture_open: true,
            controls_open: true,
            focus_info_open: false,
            open_dropdown: None,
            cameras: crate::camera::discover_cameras(),
            camera: 0,
            formats: vec!["MJPG".into(), "YUYV".into()],
            format: 0,
            resolutions: vec![
                "2592 × 1944".into(),
                "1280 × 960".into(),
                "640 × 480".into(),
            ],
            resolution: 0,
            fps_values: vec![30, 25, 20, 15, 10, 5],
            fps: 0,
            exposure: 64,
            gain: 28,
            streaming: false,
            camera_frame: None,
            current_rendered_frame: None,
            previous_rendered_frame: None,
            camera_error: None,
            capture_cancel: None,
            capture_process: None,
            recording: false,
            app_data_dir: crate::camera::default_app_data_dir(),
            sweep_recorder: None,
            captured_sweep: None,
            focus_processing: false,
            focus_progress: 0,
            focus_analysis_cancel: None,
            focus_sweep: FocusSweep::new(),
            focus_result: None,
            focus_frame: None,
            comparison_left_frame: None,
            comparison_right_frame: None,
            focus_view: FocusView::Depth,
            comparison_enabled: false,
            comparison_left: FocusView::Original,
            comparison_right: FocusView::Depth,
            comparison_split: 0.5,
            dragging_comparison: false,
            focus_resolution: FocusResolution::Width960,
            focus_region: None,
            focus_region_anchor: None,
            selecting_focus_region: false,
            tool: Tool::Pen,
            drawing: false,
            strokes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_match_best_camera_mode() {
        let app = CellSight::new();
        assert_eq!(&*app.resolutions[app.resolution], "2592 × 1944");
        assert_eq!(app.fps_values[app.fps], 30);
        assert!(!app.streaming);
        assert!(app.camera_open && app.capture_open);
    }
}
