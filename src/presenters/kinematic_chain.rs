use super::{Presenter, PresenterBuilder};
use crate::{
    controls::mouse::MouseState,
    render::{
        drawbuffer::Drawbuffer, gl_drawable::GlDrawable, gl_mesh::GlTriangleMesh,
        gl_program::GlProgram, models,
    },
    ui::widgets,
};
use egui::{widgets::DragValue, Ui};
use egui_winit::winit::dpi::PhysicalSize;
use nalgebra as na;
use std::sync::Arc;

pub struct KinematicChain {}

impl KinematicChain {
    fn new() -> Self {
        Self {}
    }
}

impl Presenter for KinematicChain {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.label("Side text");
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {}

    fn update(&mut self, delta: std::time::Duration) {}

    fn update_mouse(&mut self, state: MouseState) {}

    fn name(&self) -> &'static str {
        "Kinematic chain"
    }
}

#[derive(Default)]
pub struct KinematicChainBuilder {}

impl KinematicChainBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for KinematicChainBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("Build text")
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(KinematicChain::new())
    }
}
