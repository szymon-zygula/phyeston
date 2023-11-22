use super::Float;
use nalgebra as na;
use std::array;

pub struct Cube<T: Float>(pub [[[na::Point3<T>; 4]; 4]; 4]);

impl Cube<f64> {
    pub fn new() -> Self {
        let array = array::from_fn(|u| {
            array::from_fn(|v| {
                array::from_fn(|w| {
                    na::point![
                        2.0 * (u as f64 / 3.0) - 1.0,
                        2.0 * (v as f64 / 3.0) - 1.0,
                        2.0 * (w as f64 / 3.0) - 1.0,
                    ]
                })
            })
        });

        Self(array)
    }

    pub fn as_f32_array(&self) -> [na::Point3<f32>; 64] {
        array::from_fn(|i| {
            let u = (i / 4) / 4;
            let v = (i / 4) % 4;
            let w = (i % 4) % 4;
            self.0[u][v][w].map(|c| c as f32)
        })
    }
}
