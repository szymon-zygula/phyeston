use crate::controls::mouse::MouseState;
use egui_winit::winit::dpi::PhysicalSize;
use std::time::Duration;

pub mod jelly;
pub mod kinematic_chain;
pub mod parametrizable_function;
pub mod puma;
pub mod quaternions;
pub mod spinning_top;
pub mod spring;
pub mod hodograph;

pub trait Presenter {
    fn show_bottom_ui(&mut self, ui: &mut egui::Ui);
    fn show_side_ui(&mut self, ui: &mut egui::Ui);
    fn draw(&self, window_size: Option<PhysicalSize<u32>>);
    fn update(&mut self, delta: Duration);
    fn update_mouse(&mut self, state: MouseState);
    fn name(&self) -> &'static str;
}

pub trait PresenterBuilder {
    fn build_ui(&mut self, ui: &mut egui::Ui) -> egui::Response;
    fn build(&self, gl: std::sync::Arc<glow::Context>) -> Box<dyn Presenter>;
}
