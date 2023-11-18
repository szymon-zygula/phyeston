use super::{Presenter, PresenterBuilder};
use crate::controls::mouse::MouseState;
use egui::Ui;
use glow::HasContext;
use nalgebra as na;
use std::sync::Arc;

pub struct Jelly {
    gl: Arc<glow::Context>,
}

impl Jelly {
    const LIGHT_POSITION: na::Vector3<f32> = na::vector![-2.0, 4.0, -2.0];
    const LIGHT_COLOR: na::Vector3<f32> = na::vector![2.0, 2.0, 2.0];
    const LIGHT_AMBIENT: na::Vector3<f32> = na::vector![0.4, 0.4, 0.4];

    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self { gl }
    }
}

impl Presenter for Jelly {
    fn show_side_ui(&mut self, ui: &mut Ui) {}

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        let Some(size) = size else { return };
        let aspect_ratio = size.width as f32 / size.height as f32;
    }

    fn update(&mut self, delta: std::time::Duration) {}

    fn update_mouse(&mut self, state: MouseState) {}

    fn name(&self) -> &'static str {
        "Jelly"
    }
}

#[derive(Default)]
pub struct JellyBuilder {}

impl JellyBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for JellyBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("")
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(Jelly::new(gl))
    }
}
