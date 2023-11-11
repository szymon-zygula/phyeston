use crate::controls::mouse::MouseState;
use std::time::Duration;

pub mod parametrizable_function;
pub mod spinning_top;
pub mod spring;

pub trait Presenter {
    fn show_bottom_ui(&mut self, ui: &mut egui::Ui);
    fn show_side_ui(&mut self, ui: &mut egui::Ui);
    fn draw(&self, aspect_ratio: f32);
    fn update(&mut self, delta: Duration);
    fn update_mouse(&mut self, state: MouseState);
    fn name(&self) -> &'static str;
}

pub trait PresenterBuilder {
    fn build_ui(&mut self, ui: &mut egui::Ui);
    fn build(&self, gl: std::sync::Arc<glow::Context>) -> Box<dyn Presenter>;
}
