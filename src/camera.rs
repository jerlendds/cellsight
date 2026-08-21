use crate::app::CellSight;
use gpui::{Context, RenderImage};
use image::{Frame, ImageFormat, RgbaImage};
use smallvec::smallvec;
use std::{
    fs,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub(crate) fn default_app_data_dir() -> PathBuf {
    let directory = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cellsight");
    let _ = fs::create_dir_all(directory.join("captures"));
    directory
}

pub(crate) struct CapturedSweep {
    pub(crate) directory: PathBuf,
    pub(crate) frames: Vec<PathBuf>,
}

pub(crate) struct SweepRecorder {
    directory: PathBuf,
    sender: Option<mpsc::Sender<(usize, RgbaImage)>>,
    saved: Arc<Mutex<Vec<PathBuf>>>,
    worker: Option<thread::JoinHandle<()>>,
    next_frame: usize,
}

#[derive(Default)]
struct AnalysisStatus {
    progress: u8,
    done: bool,
    result: Option<(
        cellsight_focus_profile::FocusSweep,
        Option<cellsight_focus_profile::FocusResult>,
        Option<RgbaImage>,
        Option<RgbaImage>,
        Option<RgbaImage>,
    )>,
}

impl SweepRecorder {
    fn start(app_data_dir: &std::path::Path) -> Result<Self, String> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let directory = app_data_dir.join("captures").join(format!("sweep-{stamp}"));
        fs::create_dir_all(&directory)
            .map_err(|error| format!("Could not create capture archive: {error}"))?;
        // An unbounded handoff preserves every captured plane. Encoding runs on
        // its own thread, so temporary disk latency cannot discard Z samples.
        let (sender, receiver) = mpsc::channel::<(usize, RgbaImage)>();
        let saved = Arc::new(Mutex::new(Vec::new()));
        let worker_saved = saved.clone();
        let worker_directory = directory.clone();
        let worker = thread::spawn(move || {
            while let Ok((index, frame)) = receiver.recv() {
                let path = worker_directory.join(format!("frame-{index:06}.png"));
                if frame.save_with_format(&path, ImageFormat::Png).is_ok() {
                    worker_saved.lock().unwrap().push(path);
                }
            }
        });
        Ok(Self {
            directory,
            sender: Some(sender),
            saved,
            worker: Some(worker),
            next_frame: 0,
        })
    }

    fn record(&mut self, frame: &RgbaImage) {
        if let Some(sender) = &self.sender {
            let _ = sender.send((self.next_frame, frame.clone()));
            self.next_frame += 1;
        }
    }

    fn finish(mut self) -> CapturedSweep {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        let mut frames = self.saved.lock().unwrap().clone();
        frames.sort();
        let manifest = serde_json::json!({
            "version": 1,
            "frame_count": frames.len(),
            "frames": frames.iter().filter_map(|path| path.file_name()).map(|name| name.to_string_lossy()).collect::<Vec<_>>()
        });
        let _ = fs::write(
            self.directory.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap_or_default(),
        );
        CapturedSweep {
            directory: self.directory,
            frames,
        }
    }
}

pub(crate) fn discover_cameras() -> Vec<gpui::SharedString> {
    let mut devices: Vec<(PathBuf, String)> = fs::read_dir("/sys/class/video4linux")
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let device = PathBuf::from("/dev").join(entry.file_name());
            if !device.exists() {
                return None;
            }
            let name = fs::read_to_string(entry.path().join("name"))
                .unwrap_or_else(|_| "Video device".to_owned());
            Some((device, name.trim().to_owned()))
        })
        .collect();
    devices.sort_by(|a, b| a.0.cmp(&b.0));
    if devices.is_empty() {
        vec!["No video devices detected".into()]
    } else {
        devices
            .into_iter()
            .map(|(path, name)| format!("{name} · {}", path.display()).into())
            .collect()
    }
}

impl CellSight {
    pub(crate) fn toggle_focus_sweep(&mut self, cx: &mut Context<Self>) {
        if self.recording {
            self.recording = false;
            if let Some(recorder) = self.sweep_recorder.take() {
                self.captured_sweep = Some(recorder.finish());
            }
            self.comparison_enabled = true;
            self.rerun_captured_sweep(cx);
        } else {
            if let Some(cancel) = self.focus_analysis_cancel.take() {
                cancel.store(true, Ordering::Relaxed);
            }
            self.focus_processing = false;
            self.selecting_focus_region = false;
            self.focus_region_anchor = None;
            self.focus_sweep.clear();
            self.focus_result = None;
            self.focus_frame = None;
            self.comparison_left_frame = None;
            self.comparison_right_frame = None;
            match SweepRecorder::start(&self.app_data_dir) {
                Ok(recorder) => {
                    self.sweep_recorder = Some(recorder);
                    self.recording = true;
                }
                Err(error) => self.camera_error = Some(error.into()),
            }
        }
    }

    pub(crate) fn set_focus_resolution(
        &mut self,
        resolution: cellsight_focus_profile::FocusResolution,
        cx: &mut Context<Self>,
    ) {
        if self.recording {
            return;
        }
        self.focus_resolution = resolution;
        self.rerun_captured_sweep(cx);
    }

    pub(crate) fn rerun_captured_sweep(&mut self, cx: &mut Context<Self>) {
        if self.recording {
            return;
        }
        let Some(frames) = self
            .captured_sweep
            .as_ref()
            .map(|capture| capture.frames.clone())
        else {
            return;
        };
        if let Some(cancel) = self.focus_analysis_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        let resolution = self.focus_resolution;
        let region = self.focus_region;
        let view = self.focus_view;
        let comparison_left = self.comparison_left;
        let comparison_right = self.comparison_right;
        let cancel = Arc::new(AtomicBool::new(false));
        self.focus_analysis_cancel = Some(cancel.clone());
        self.focus_processing = true;
        self.focus_progress = 0;
        let status = Arc::new(Mutex::new(AnalysisStatus::default()));
        let worker_status = status.clone();
        let worker_cancel = cancel.clone();
        thread::spawn(move || {
            let total = frames.len().max(1);
            let mut sweep = cellsight_focus_profile::FocusSweep::new();
            for (index, path) in frames.into_iter().enumerate() {
                if worker_cancel.load(Ordering::Relaxed) {
                    return;
                }
                if let Ok(frame) = image::open(path) {
                    sweep.push_region(&frame.into_rgba8(), resolution, region);
                }
                worker_status.lock().unwrap().progress = (((index + 1) * 90 / total).min(90)) as u8;
            }
            let result = sweep.finish();
            worker_status.lock().unwrap().progress = 95;
            let rendered = result.as_ref().map(|result| result.render(view));
            let left = result.as_ref().map(|result| result.render(comparison_left));
            let right = result
                .as_ref()
                .map(|result| result.render(comparison_right));
            let mut status = worker_status.lock().unwrap();
            status.progress = 100;
            status.result = Some((sweep, result, rendered, left, right));
            status.done = true;
        });
        cx.spawn(async move |this, cx| {
            loop {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let (progress, done, result) = {
                    let mut status = status.lock().unwrap();
                    (status.progress, status.done, status.result.take())
                };
                let disconnected = this
                    .update(cx, |this, cx| {
                        if cancel.load(Ordering::Relaxed) {
                            return;
                        }
                        this.focus_progress = progress;
                        if done {
                            if let Some((sweep, result, rendered, left, right)) = result {
                                this.focus_sweep = sweep;
                                this.focus_result = result;
                                this.focus_frame = rendered.map(|image| {
                                    Arc::new(RenderImage::new(smallvec![Frame::new(image)]))
                                });
                                this.comparison_left_frame = left.map(|image| {
                                    Arc::new(RenderImage::new(smallvec![Frame::new(image)]))
                                });
                                this.comparison_right_frame = right.map(|image| {
                                    Arc::new(RenderImage::new(smallvec![Frame::new(image)]))
                                });
                            }
                            this.focus_processing = false;
                            this.focus_analysis_cancel = None;
                        }
                        cx.notify();
                    })
                    .is_err();
                if disconnected || done {
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(32))
                    .await;
            }
        })
        .detach();
    }

    pub(crate) fn set_comparison_left(&mut self, view: cellsight_focus_profile::FocusView) {
        self.comparison_left = view;
        self.render_focus_views();
    }

    pub(crate) fn set_comparison_right(&mut self, view: cellsight_focus_profile::FocusView) {
        self.comparison_right = view;
        self.focus_view = view;
        self.render_focus_views();
    }

    fn render_focus_views(&mut self) {
        self.comparison_left_frame = self.focus_result.as_ref().map(|result| {
            Arc::new(RenderImage::new(smallvec![Frame::new(
                result.render(self.comparison_left)
            )]))
        });
        self.comparison_right_frame = self.focus_result.as_ref().map(|result| {
            Arc::new(RenderImage::new(smallvec![Frame::new(
                result.render(self.comparison_right)
            )]))
        });
        self.focus_frame = self.comparison_right_frame.clone();
    }

    pub(crate) fn start_camera(&mut self, cx: &mut Context<Self>) {
        self.stop_camera();
        self.camera_error = None;
        let previous_device = self
            .cameras
            .get(self.camera)
            .and_then(|camera| camera.split('·').nth(1).map(str::trim).map(str::to_owned));
        self.cameras = discover_cameras();
        self.camera = previous_device
            .and_then(|path| {
                self.cameras
                    .iter()
                    .position(|camera| camera.ends_with(&path))
            })
            .unwrap_or(0);
        let Some(device) = self.cameras.get(self.camera).and_then(|camera| {
            camera
                .split('·')
                .nth(1)
                .map(str::trim)
                .filter(|path| path.starts_with("/dev/video"))
                .map(str::to_owned)
        }) else {
            self.camera_error = Some(
                "No camera detected. Reconnect the microscope, then click Connect camera again."
                    .into(),
            );
            cx.notify();
            return;
        };
        self.streaming = true;
        let resolution = self.resolutions[self.resolution]
            .replace('×', "x")
            .replace(' ', "");
        let (source_width, source_height) = resolution
            .split_once('x')
            .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
            .unwrap_or((1280_u32, 960_u32));
        // Preserve the camera's native pixels: these frames are also the source
        // archive for later high-resolution depth reconstruction.
        let preview_width = source_width;
        let preview_height = source_height;
        let frame_byte_len = preview_width as usize * preview_height as usize * 4;
        let fps = self.fps_values[self.fps].to_string();
        let format = match self.formats[self.format].as_ref() {
            "MJPG" => "mjpeg",
            "YUYV" => "yuyv422",
            other => other,
        }
        .to_owned();
        let cancel = Arc::new(AtomicBool::new(false));
        let consumer_cancel = cancel.clone();
        self.capture_cancel = Some(cancel.clone());
        // A single replaceable slot ensures a slow renderer never works through
        // stale camera frames. Producers always replace the pending frame.
        let latest = Arc::new(Mutex::new(None));
        let producer_latest = latest.clone();
        let mut command = Command::new("ffmpeg");
        command.args(["-hide_banner", "-loglevel", "error"]);
        command.args([
            "-fflags",
            "nobuffer",
            "-flags",
            "low_delay",
            "-analyzeduration",
            "0",
            "-probesize",
            "32",
        ]);
        command.args([
            "-f",
            "v4l2",
            "-input_format",
            &format,
            "-video_size",
            &resolution,
            "-framerate",
            &fps,
            "-i",
            &device,
            "-an",
            "-pix_fmt",
            "rgba",
            "-f",
            "rawvideo",
            "-flush_packets",
            "1",
        ]);
        let mut child = match command
            .arg("-")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                self.streaming = false;
                self.capture_cancel = None;
                self.camera_error = Some(format!("Could not start ffmpeg: {error}").into());
                cx.notify();
                return;
            }
        };
        let mut stdout = child.stdout.take().expect("ffmpeg stdout was piped");
        let process = Arc::new(Mutex::new(child));
        self.capture_process = Some(process.clone());

        thread::spawn(move || {
            while !cancel.load(Ordering::Relaxed) {
                let mut bytes = vec![0_u8; frame_byte_len];
                match stdout.read_exact(&mut bytes) {
                    Ok(()) => {
                        let frame =
                            image::RgbaImage::from_raw(preview_width, preview_height, bytes)
                                .ok_or_else(|| "Invalid raw camera frame size".to_owned());
                        // Replacing this slot drops frames the UI did not have
                        // time to paint instead of allowing latency to grow.
                        *producer_latest.lock().unwrap() = Some(frame);
                    }
                    Err(error) => {
                        *producer_latest.lock().unwrap() =
                            Some(Err(format!("Camera read failed: {error}")));
                        break;
                    }
                }
            }
            let mut child = process.lock().unwrap();
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
            } else {
                let mut details = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut details);
                }
                let message = if details.trim().is_empty() {
                    "Camera stream ended before a frame was received".to_owned()
                } else {
                    details.trim().to_owned()
                };
                *producer_latest.lock().unwrap() = Some(Err(message));
            }
            let _ = child.wait();
        });
        cx.spawn(async move |this, cx| {
            loop {
                // A restart replaces the cancellation token. Stop this task
                // before it can publish an error or frame from the old stream.
                if consumer_cancel.load(Ordering::Relaxed) {
                    break;
                }
                let newest = latest.lock().unwrap().take();
                if let Some(message) = newest {
                    let disconnected = this
                        .update(cx, |this, cx| {
                            if consumer_cancel.load(Ordering::Relaxed) || !this.streaming {
                                return;
                            }
                            match message {
                                Ok(rgba) => {
                                    if this.recording {
                                        if let Some(recorder) = &mut this.sweep_recorder {
                                            recorder.record(&rgba);
                                        }
                                        this.focus_sweep.push_region(
                                            &rgba,
                                            this.focus_resolution,
                                            this.focus_region,
                                        );
                                    }
                                    this.camera_frame = Some(Arc::new(RenderImage::new(
                                        smallvec![Frame::new(rgba)],
                                    )));
                                    this.camera_error = None;
                                }
                                Err(error) => {
                                    this.camera_error = Some(error.into());
                                    this.streaming = false;
                                }
                            }
                            cx.notify();
                        })
                        .is_err();
                    if disconnected {
                        break;
                    }
                }
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
            }
        })
        .detach();
    }

    pub(crate) fn stop_camera(&mut self) {
        if let Some(cancel) = self.capture_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        if let Some(process) = self.capture_process.take()
            && let Ok(mut child) = process.lock()
        {
            // Interrupt a blocking stdout read and release /dev/video immediately
            // so a following Connect can acquire it reliably.
            let _ = child.kill();
            let _ = child.wait();
        }
        self.streaming = false;
    }
}
