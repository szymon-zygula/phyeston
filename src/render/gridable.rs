use super::mesh::ClassicVertex;
use crate::numerics::parametric::ParametricForm;
use nalgebra as na;

pub trait Gridable {
    fn grid(&self, points_x: u32, points_y: u32) -> (Vec<ClassicVertex>, Vec<u32>);
}

impl<T: ParametricForm<2, 3>> Gridable for T {
    fn grid(&self, points_x: u32, points_y: u32) -> (Vec<ClassicVertex>, Vec<u32>) {
        let point_count = (points_x + 1) * (points_y + 1);
        let mut points = Vec::with_capacity(point_count as usize);
        let mut indices = Vec::with_capacity(2 * point_count as usize);

        for x_idx in 0..(points_x + 1) {
            for y_idx in 0..(points_y + 1) {
                let x_range = self.bounds().x.1 - self.bounds().x.0;
                let x = x_idx as f64 / points_x as f64 * x_range + self.bounds().x.0;

                let y_range = self.bounds().y.1 - self.bounds().y.0;
                let y = y_idx as f64 / points_y as f64 * y_range + self.bounds().y.0;

                let position = self.value(&na::Vector2::new(x, y));
                let normal = self.normal(&na::Vector2::new(x, y));
                let point_idx = points.len() as u32;
                points.push(ClassicVertex {
                    position: position.map(|c| c as f32),
                    normal: normal.map(|c| c as f32),
                });

                indices.push(point_idx);
                indices.push((y_idx + 1) % (points_y + 1) + x_idx * (points_y + 1));
                indices.push(point_idx);
                indices.push((point_idx + points_y + 1) % point_count);
            }
        }

        (points, indices)
    }
}
