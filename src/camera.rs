use crate::app::CellSight;
use gpui::{Context, RenderImage};
use image::Frame;
use smallvec::smallvec;
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

impl CellSight {
    pub(crate) fn start_camera(&mut self, cx: &mut Context<Self>) {
        self.stop_camera();
        self.streaming = true;
        self.camera_error = None;
        let device = self.cameras[self.camera]
            .split('·')
            .nth(1)
            .map(str::trim)
            .filter(|path| path.starts_with("/dev/video"))
            .unwrap_or("/dev/video0")
            .to_owned();
        let resolution = self.resolutions[self.resolution]
            .replace('×', "x")
            .replace(' ', "");
        let (source_width, source_height) = resolution
            .split_once('x')
            .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
            .unwrap_or((1280_u32, 960_u32));
        // Uploading a new 20 MB texture thirty times a second is unnecessary
        // for the on-screen preview. Keep full resolution at the camera, but
        // let ffmpeg's optimized scaler produce a viewport-sized stream.
        let preview_width = source_width.min(1280);
        let preview_height = (source_height * preview_width / source_width).max(1);
        let preview_size = format!("{preview_width}:{preview_height}");
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
            "-vf",
            &format!("scale={preview_size}:flags=fast_bilinear"),
            "-pix_fmt",
            "bgra",
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
