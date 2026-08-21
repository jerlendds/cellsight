//! Focus-from-defocus analysis for a monotonic microscope focus sweep.

use image::{Rgba, RgbaImage, imageops::FilterType};

const MAX_PLANES: usize = 180;
const MAX_STACK_BYTES: usize = 512 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusResolution {
    Width320,
    Width640,
    Width960,
    Full,
}

impl FocusResolution {
    pub fn target_width(self) -> Option<u32> {
        match self {
            Self::Width320 => Some(320),
            Self::Width640 => Some(640),
            Self::Width960 => Some(960),
            Self::Full => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusView {
    Original,
    Depth,
    Falloff,
    Confidence,
    AllInFocus,
}

#[derive(Debug)]
pub struct FocusResult {
    pub width: u32,
    pub height: u32,
    pub planes: usize,
    pub plane_limited: bool,
    pub depth: Vec<f32>,
    pub falloff: Vec<f32>,
    pub confidence: Vec<f32>,
    original: RgbaImage,
    all_in_focus: RgbaImage,
}

impl FocusResult {
    pub fn render(&self, view: FocusView) -> RgbaImage {
        if view == FocusView::Original {
            return self.original.clone();
        }
        if view == FocusView::AllInFocus {
            return self.all_in_focus.clone();
        }
        let mut out = RgbaImage::new(self.width, self.height);
        for (index, pixel) in out.pixels_mut().enumerate() {
            let value = match view {
                FocusView::Depth => turbo(self.depth[index]),
                FocusView::Falloff => turbo(self.falloff[index]),
                FocusView::Confidence => {
                    let c = (self.confidence[index] * 255.0) as u8;
                    [c, c, c]
                }
                FocusView::AllInFocus => unreachable!(),
                FocusView::Original => unreachable!(),
            };
            *pixel = Rgba([value[0], value[1], value[2], 255]);
        }
        out
    }
}

#[derive(Default, Debug)]
pub struct FocusSweep {
    width: u32,
    height: u32,
    frames: Vec<RgbaImage>,
    sharpness: Vec<Vec<f32>>,
    reference_gray: Option<Vec<f32>>,
    plane_limited: bool,
}

impl FocusSweep {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Adds one plane. Frame order is used as relative Z when hardware Z is absent.
    pub fn push(&mut self, frame: &RgbaImage) {
        self.push_region(frame, FocusResolution::Width320, None);
    }

    /// Adds a plane at the requested analysis resolution. `region` is a
    /// normalized `(left, top, right, bottom)` crop in source-frame space.
    pub fn push_region(
        &mut self,
        frame: &RgbaImage,
        resolution: FocusResolution,
        region: Option<[f32; 4]>,
    ) {
        let cropped = crop_normalized(frame, region);
        let (width, height) = analysis_size(cropped.width(), cropped.height(), resolution);
        let bytes_per_plane = width as usize * height as usize * 8;
        let plane_limit = (MAX_STACK_BYTES / bytes_per_plane.max(1)).clamp(3, MAX_PLANES);
        if self.frames.len() >= plane_limit {
            self.plane_limited = true;
            return;
        }
        if self.is_empty() {
            self.width = width;
            self.height = height;
        }
        let resized =
            image::imageops::resize(&cropped, self.width, self.height, FilterType::Triangle);
        let gray = grayscale(&resized);
        let aligned = if let Some(reference) = &self.reference_gray {
            let (dx, dy) = estimate_translation(reference, &gray, self.width, self.height);
            translate(&resized, dx, dy)
        } else {
            self.reference_gray = Some(gray);
            resized
        };
        self.sharpness.push(local_sharpness(&aligned));
        self.frames.push(aligned);
    }

    pub fn finish(&self) -> Option<FocusResult> {
        if self.frames.len() < 3 {
            return None;
        }
        let pixels = (self.width * self.height) as usize;
        let last = (self.frames.len() - 1) as f32;
        let mut depth = vec![0.0; pixels];
        let mut falloff = vec![0.0; pixels];
        let mut confidence = vec![0.0; pixels];
        let mut all_in_focus = RgbaImage::new(self.width, self.height);
        for p in 0..pixels {
            let (best_z, best) = self
                .sharpness
                .iter()
                .enumerate()
                .map(|(z, plane)| (z, plane[p]))
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();
            let background =
                self.sharpness.iter().map(|plane| plane[p]).sum::<f32>() / self.frames.len() as f32;
            let left = self.sharpness[best_z.saturating_sub(1)][p];
            let right = self.sharpness[(best_z + 1).min(self.frames.len() - 1)][p];
            let denominator = left - 2.0 * best + right;
            let sub_plane =
                if best_z > 0 && best_z + 1 < self.frames.len() && denominator.abs() > 1e-6 {
                    (0.5 * (left - right) / denominator).clamp(-0.5, 0.5)
                } else {
                    0.0
                };
            depth[p] = ((best_z as f32 + sub_plane) / last).clamp(0.0, 1.0);
            confidence[p] = (best / (best + background + 1e-6)).clamp(0.0, 1.0);
            // Discrete negative curvature: narrow focus peaks produce high values.
            falloff[p] = ((2.0 * best - left - right) / (best + 1e-6)).clamp(0.0, 1.0);
            let x = p as u32 % self.width;
            let y = p as u32 / self.width;
            all_in_focus.put_pixel(x, y, *self.frames[best_z].get_pixel(x, y));
        }
        normalize_contrast(&mut falloff);
        normalize_contrast(&mut confidence);
        Some(FocusResult {
            width: self.width,
            height: self.height,
            planes: self.frames.len(),
            plane_limited: self.plane_limited,
            depth,
            falloff,
            confidence,
            original: self.frames[self.frames.len() / 2].clone(),
            all_in_focus,
        })
    }
}

fn analysis_size(width: u32, height: u32, resolution: FocusResolution) -> (u32, u32) {
    let Some(target_width) = resolution.target_width() else {
        return (width, height);
    };
    if width <= target_width {
        return (width, height);
    }
    (target_width, (height * target_width / width).max(1))
}

fn crop_normalized(frame: &RgbaImage, region: Option<[f32; 4]>) -> RgbaImage {
    let Some([left, top, right, bottom]) = region else {
        return frame.clone();
    };
    let x = (left.clamp(0.0, 1.0) * frame.width() as f32) as u32;
    let y = (top.clamp(0.0, 1.0) * frame.height() as f32) as u32;
    let right = (right.clamp(0.0, 1.0) * frame.width() as f32) as u32;
    let bottom = (bottom.clamp(0.0, 1.0) * frame.height() as f32) as u32;
    image::imageops::crop_imm(
        frame,
        x,
        y,
        right.saturating_sub(x).max(1),
        bottom.saturating_sub(y).max(1),
    )
    .to_image()
}

fn local_sharpness(image: &RgbaImage) -> Vec<f32> {
    let (w, h) = image.dimensions();
    let gray = grayscale(image);
    let mut score = vec![0.0; gray.len()];
    if w < 3 || h < 3 {
        return score;
    }
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = (y * w + x) as usize;
            let gx = gray[i + 1] - gray[i - 1];
            let gy = gray[i + w as usize] - gray[i - w as usize];
            score[i] = gx * gx + gy * gy;
        }
    }
    // A small box filter makes the estimate local-region based rather than noisy per-pixel.
    box_blur(&score, w, h, 2)
}

fn grayscale(image: &RgbaImage) -> Vec<f32> {
    image
        .pixels()
        .map(|p| 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32)
        .collect()
}

/// Registers lateral camera/sample motion to the first plane using a robust,
/// sampled integer translation search. Alignment happens before focus scoring.
fn estimate_translation(reference: &[f32], current: &[f32], w: u32, h: u32) -> (i32, i32) {
    if w < 16 || h < 16 {
        return (0, 0);
    }
    let step = ((((w as usize * h as usize) / 20_000).max(4) as f32).sqrt() as usize).max(1);
    let mut best = (f64::INFINITY, 0, 0);
    for dy in -6_i32..=6 {
        for dx in -6_i32..=6 {
            let mut error = 0.0_f64;
            let mut count = 0_usize;
            for y in (8..h as usize - 8).step_by(step) {
                for x in (8..w as usize - 8).step_by(step) {
                    let shifted_x = (x as i32 + dx) as usize;
                    let shifted_y = (y as i32 + dy) as usize;
                    let difference =
                        reference[y * w as usize + x] - current[shifted_y * w as usize + shifted_x];
                    error += (difference * difference) as f64;
                    count += 1;
                }
            }
            let mean = error / count.max(1) as f64;
            if mean < best.0 {
                best = (mean, dx, dy);
            }
        }
    }
    (best.1, best.2)
}

fn translate(image: &RgbaImage, dx: i32, dy: i32) -> RgbaImage {
    if dx == 0 && dy == 0 {
        return image.clone();
    }
    let (w, h) = image.dimensions();
    let mut aligned = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let source_x = (x as i32 + dx).clamp(0, w as i32 - 1) as u32;
            let source_y = (y as i32 + dy).clamp(0, h as i32 - 1) as u32;
            aligned.put_pixel(x, y, *image.get_pixel(source_x, source_y));
        }
    }
    aligned
}

fn box_blur(input: &[f32], w: u32, h: u32, radius: u32) -> Vec<f32> {
    let mut out = vec![0.0; input.len()];
    let stride = w as usize + 1;
    let mut integral = vec![0.0; stride * (h as usize + 1)];
    for y in 0..h as usize {
        let mut row_sum = 0.0;
        for x in 0..w as usize {
            row_sum += input[y * w as usize + x];
            integral[(y + 1) * stride + x + 1] = integral[y * stride + x + 1] + row_sum;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let x0 = x.saturating_sub(radius) as usize;
            let y0 = y.saturating_sub(radius) as usize;
            let x1 = ((x + radius).min(w - 1) + 1) as usize;
            let y1 = ((y + radius).min(h - 1) + 1) as usize;
            let sum = integral[y1 * stride + x1]
                - integral[y0 * stride + x1]
                - integral[y1 * stride + x0]
                + integral[y0 * stride + x0];
            out[(y * w + x) as usize] = sum / ((x1 - x0) * (y1 - y0)) as f32;
        }
    }
    out
}

fn normalize_contrast(values: &mut [f32]) {
    let max = values.iter().copied().fold(0.0_f32, f32::max);
    if max > 0.0 {
        values.iter_mut().for_each(|v| *v = (*v / max).sqrt());
    }
}

fn turbo(x: f32) -> [u8; 3] {
    // Compact blue → cyan → yellow → red scientific colour map.
    let x = x.clamp(0.0, 1.0);
    let r = (1.5 - (4.0 * x - 3.0).abs()).clamp(0.0, 1.0);
    let g = (1.5 - (4.0 * x - 2.0).abs()).clamp(0.0, 1.0);
    let b = (1.5 - (4.0 * x - 1.0).abs()).clamp(0.0, 1.0);
    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_different_peak_planes() {
        let mut sweep = FocusSweep::new();
        for z in 0..5 {
            let mut frame = RgbaImage::from_pixel(12, 8, Rgba([100, 100, 100, 255]));
            if z == 1 {
                for y in 1..7 {
                    for x in 1..6 {
                        if (x + y) % 2 == 0 {
                            frame.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                        }
                    }
                }
            }
            if z == 3 {
                for y in 1..7 {
                    for x in 6..11 {
                        if (x + y) % 2 == 0 {
                            frame.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                        }
                    }
                }
            }
            sweep.push(&frame);
        }
        let result = sweep.finish().unwrap();
        assert!(result.depth[(4 * 12 + 3) as usize] < result.depth[(4 * 12 + 8) as usize]);
        assert_eq!(result.planes, 5);
    }

    #[test]
    fn needs_three_planes() {
        let mut sweep = FocusSweep::new();
        sweep.push(&RgbaImage::new(4, 4));
        assert!(sweep.finish().is_none());
    }

    #[test]
    fn honors_resolution_and_normalized_region() {
        let frame = RgbaImage::new(1200, 800);
        let mut sweep = FocusSweep::new();
        for _ in 0..3 {
            sweep.push_region(
                &frame,
                FocusResolution::Width640,
                Some([0.25, 0.25, 0.75, 0.75]),
            );
        }
        let result = sweep.finish().unwrap();
        // The 600px-wide crop is not upscaled to the 640 stop.
        assert_eq!((result.width, result.height), (600, 400));
    }
}
