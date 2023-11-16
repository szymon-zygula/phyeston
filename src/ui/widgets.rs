use egui::{containers::Frame, emath::Numeric, *};
use nalgebra as na;

pub fn vector_drag<T: Numeric, const S: usize>(
    ui: &mut Ui,
    vec: &mut na::SVector<T, S>,
    min: T,
    max: T,
    suffix: &str,
    speed: f64,
    coords: &[&str; S],
) -> Response {
    Frame::none()
        .stroke(Stroke::new(0.5, Color32::GRAY))
        .rounding(4.0)
        .inner_margin(5.0)
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                (0..S)
                    .map(|i| {
                        ui.label(coords[i]);
                        ui.add(
                            DragValue::new(&mut vec[i])
                                .clamp_range(min..=max)
                                .suffix(suffix)
                                .speed(speed),
                        )
                    })
                    .reduce(|a, b| a | b)
                    .unwrap()
            })
            .inner
        })
        .inner
}
