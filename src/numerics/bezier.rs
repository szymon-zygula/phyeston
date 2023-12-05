use super::Float;
use nalgebra as na;
use std::array;

pub struct Cube<F: Float>(pub [[[na::Point3<F>; 4]; 4]; 4]);

impl<F: Float> Cube<F> {
    pub fn as_flat(&self) -> [F; 3 * 64] {
        array::from_fn(|i| {
            let u = i / 3 / 4 / 4;
            let v = (i / 3 / 4) % 4;
            let w = (i / 3) % 4;
            let c = i % 3;
            self.0[u][v][w][c]
        })
    }

    fn flat_idx(i: usize) -> (usize, usize, usize) {
        let u = (i / 4) / 4;
        let v = (i / 4) % 4;
        let w = i % 4;
        (u, v, w)
    }

    pub fn flat(&self, i: usize) -> &na::Point3<F> {
        let indices = Self::flat_idx(i);
        &self.0[indices.0][indices.1][indices.2]
    }

    pub fn flat_mut(&mut self, i: usize) -> &mut na::Point3<F> {
        let indices = Self::flat_idx(i);
        &mut self.0[indices.0][indices.1][indices.2]
    }
}

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
        array::from_fn(|i| self.flat(i).map(|c| c as f32))
    }

    pub fn as_f32_flat(&self) -> [f32; 3 * 64] {
        self.as_flat().map(|c| c as f32)
    }

    pub fn patches_f32(&self) -> [[[na::Point3<f32>; 4]; 4]; 6] {
        [
            self.0[3].map(|v| v.map(|w| w.map(|c| c as f32))),
            self.0[3].map(|v| v.map(|w| w.map(|c| c as f32))),
            self.0[3].map(|v| v.map(|w| w.map(|c| c as f32))),
            self.0[0].map(|v| v.map(|w| w.map(|c| c as f32))),
            self.0[3].map(|v| v.map(|w| w.map(|c| c as f32))),
            self.0[3].map(|v| v.map(|w| w.map(|c| c as f32))),
        ]
    }
}
