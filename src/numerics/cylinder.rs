use super::parametric::ParametricForm;
use crate::render::{
    gridable::Triangable,
    mesh::{ClassicVertex, Triangle},
};
use itertools::Itertools;
use nalgebra as na;

#[derive(Copy, Clone, Debug)]
pub struct Cylinder {
    pub radius: f64,
    pub length: f64,
}

impl Cylinder {
    pub fn new(radius: f64, length: f64) -> Self {
        Self { radius, length }
    }

    fn r_at(&self, vec: &na::Vector2<f64>) -> f64 {
        self.radius
            * if vec.y < 0.0 {
                10.0 * (vec.y + 0.1)
            } else if vec.y > 1.0 {
                10.0 * (1.1 - vec.y)
            } else {
                1.0
            }
    }
}

impl ParametricForm<2, 3> for Cylinder {
    fn bounds(&self) -> na::Vector2<(f64, f64)> {
        na::Vector2::new(
            (0.0, 2.0 * std::f64::consts::PI),
            (-0.1, 1.1), // [0.0, 1.0] for the walls, the rest for the tops
        )
    }

    fn wrapped(&self, dim: usize) -> bool {
        dim == 0
    }

    fn value(&self, vec: &na::Vector2<f64>) -> na::Point3<f64> {
        let r = self.r_at(vec);

        na::Point3::new(
            r * vec.x.cos(),
            r * vec.x.sin(),
            self.length * vec.y.clamp(0.0, 1.0),
        )
    }

    fn normal(&self, vec: &na::Vector2<f64>) -> na::Vector3<f64> {
        (if vec.y < 0.0 {
            na::vector![0.0, 0.0, -1.0]
        } else if vec.y > 1.0 {
            na::vector![0.0, 0.0, 1.0]
        } else {
            na::vector!(vec.x.cos(), vec.x.sin(), 0.0)
        })
        .normalize()
    }
}

impl Triangable for Cylinder {
    fn triangulation(
        &self,
        points_x: u32,
        _points_y: u32,
    ) -> (
        Vec<crate::render::mesh::ClassicVertex>,
        Vec<crate::render::mesh::Triangle>,
    ) {
        let mut vertices = Vec::new();

        // Top
        let top_center_idx = vertices.len() as u32;
        vertices.push(ClassicVertex::new(
            na::point![0.0, 0.0, 1.0],
            na::vector![0.0, 0.0, 1.0],
        ));
        for i in 0..points_x {
            let t = i as f32 / (points_x - 1) as f32 * std::f32::consts::PI * 2.0;
            let position = na::point![t.cos(), t.sin(), 1.0];
            let normal = na::vector![0.0, 0.0, 1.0];
            vertices.push(ClassicVertex::new(position, normal));
        }

        // Bottom
        let bottom_center_idx = vertices.len() as u32;
        vertices.push(ClassicVertex::new(
            na::point![0.0, 0.0, -1.0],
            na::vector![0.0, 0.0, -1.0],
        ));
        for i in 0..points_x {
            let t = i as f32 / (points_x - 1) as f32 * std::f32::consts::PI * 2.0;
            let position = na::point![t.cos(), t.sin(), -1.0];
            let normal = na::vector![0.0, 0.0, -1.0];
            vertices.push(ClassicVertex::new(position, normal));
        }

        // Side top
        let sides_top_idx = vertices.len() as u32;
        for i in 0..points_x {
            let t = i as f32 / (points_x - 1) as f32 * std::f32::consts::PI * 2.0;
            let position = na::point![t.cos(), t.sin(), 1.0];
            let normal = na::vector![t.cos(), t.sin(), 0.0];
            vertices.push(ClassicVertex::new(position, normal));
        }

        // Side bottom
        let sides_bottom_idx = vertices.len() as u32;
        for i in 0..points_x {
            let t = (i as f32 / (points_x - 1) as f32) * std::f32::consts::PI * 2.0;
            let position = na::point![t.cos(), t.sin(), -1.0];
            let normal = na::vector![t.cos(), t.sin(), 0.0];
            vertices.push(ClassicVertex::new(position, normal));
        }

        let mut triangles = Vec::new();

        for (i, j) in (0..points_x as u32).chain([0]).tuple_windows() {
            triangles.push(Triangle([
                i + top_center_idx + 1,
                j + top_center_idx + 1,
                top_center_idx,
            ]));

            triangles.push(Triangle([
                j + bottom_center_idx + 1,
                i + bottom_center_idx + 1,
                bottom_center_idx,
            ]));
        }

        for (i, j) in (0..points_x as u32).chain([0]).tuple_windows() {
            triangles.push(Triangle([
                j + sides_top_idx,
                i + sides_top_idx,
                i + sides_bottom_idx,
            ]));

            triangles.push(Triangle([
                i + sides_bottom_idx,
                j + sides_bottom_idx,
                j + sides_top_idx,
            ]));
        }

        (vertices, triangles)
    }
}
