mod sidebar;
mod theme;
mod toolbar;
mod viewport;

use crate::app::CellSight;
use gpui::{Context, IntoElement, Render, Window, div, prelude::*, rgb};
use theme::TEXT;

impl Render for CellSight {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // RenderImage textures are keyed by a unique ID in GPUI's sprite atlas.
        // Camera frames therefore have to be retired explicitly or the atlas
        // eventually fills and leaves the viewport displaying a stale frame.
        if let Some(latest_frame) = self.camera_frame.clone()
            && self.current_rendered_frame.as_ref().map(|frame| frame.id) != Some(latest_frame.id)
        {
            if let Some(current_frame) = self.current_rendered_frame.take() {
                if let Some(previous_frame) = self.previous_rendered_frame.take()
                    && previous_frame.id != current_frame.id
                {
                    let _ = window.drop_image(previous_frame);
                }
                self.previous_rendered_frame = Some(current_frame);
            }
            self.current_rendered_frame = Some(latest_frame);
        }

        let toolbar = toolbar::render(self, cx);
        let sidebar = sidebar::render(self, cx);
        let viewport = viewport::render(self, cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x0d1116))
            .text_color(rgb(TEXT))
            .font_family("Neometric")
            .child(toolbar)
            .child(if self.focus_processing {
                div()
                    .relative()
                    .h(gpui::px(22.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px_4()
                    .bg(rgb(0x11151a))
                    .text_xs()
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .left_0()
                            .h(gpui::px(3.))
                            .w(gpui::relative(self.focus_progress as f32 / 100.0))
                            .bg(rgb(0x5cc8ff)),
                    )
                    .child(format!(
                        "Processing depth profile… {}%",
                        self.focus_progress
                    ))
            } else {
                div()
            })
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(sidebar)
                    .child(viewport),
            )
    }
}
