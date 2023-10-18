pub mod parametrizable_function;
pub mod spring;

pub trait Presenter {
    fn show_bottom_ui(&mut self, ui: &mut egui::Ui);
    fn show_side_ui(&mut self, ui: &mut egui::Ui);
    fn draw(&self, aspect_ratio: f32);
    fn update(&mut self);
    fn name(&self) -> &'static str;
}

pub trait PresenterBuilder {
    type Target;
    fn build_ui(&mut self, ui: &mut egui::Ui);
    fn build(&self, gl: std::sync::Arc<glow::Context>) -> Self::Target;
}
