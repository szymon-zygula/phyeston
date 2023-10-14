pub mod spring;
pub mod parametrizable_function;

pub trait Presenter {
    fn show_bottom_ui(&mut self, ui: &mut egui::Ui);
    fn show_side_ui(&mut self, ui: &mut egui::Ui);
    fn draw(&self, aspect_ratio: f32);
    fn update(&mut self);
    fn name(&self) -> &'static str;
}
