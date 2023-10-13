pub mod spring;

pub trait Presenter {
    fn show_bottom_ui(&self, ui: &mut egui::Ui);
    fn show_side_ui(&self, ui: &mut egui::Ui);
    fn draw(&self);
    fn update(&mut self);
    fn name(&self) -> &'static str;
}
