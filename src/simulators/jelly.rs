use crate::numerics::{
    bezier,
    ode::{PlainODE, State},
};
use itertools::Itertools;
use nalgebra as na;
use std::array;

pub const POINT_COUNT: usize = 64;
pub const SPACE_DIM: usize = POINT_COUNT * 3;
pub const ODE_DIM: usize = SPACE_DIM * 2;

pub type JellyState = State<ODE_DIM>;

pub struct JellyODE {
    pub masses: [[[f64; 4]; 4]; 4],
    pub corner_spring_constant: f64,
    pub inner_spring_constant: f64,
}

impl JellyODE {
    pub fn new() -> Self {
        let masses = array::from_fn(|u| array::from_fn(|v| array::from_fn(|w| 1.0)));

        Self {
            masses,
            corner_spring_constant: 1.0,
            inner_spring_constant: 1.0,
        }
    }

    pub fn default_state() -> JellyState {
        JellyState {
            t: 0.0,
            y: na::SVector::from_iterator(
                bezier::Cube::new()
                    .as_flat()
                    .iter()
                    .chain([0.0].iter().cycle().take(SPACE_DIM))
                    .copied(),
            ),
        }
    }

    fn force(&self, u: usize, v: usize, w: usize) -> na::Vector3<f64> {
        na::vector![0.0, 0.0, 0.0]
    }

    fn accelerations(&self) -> na::SVector<f64, SPACE_DIM> {
        na::SVector::from_iterator(
            (0..4)
                .cartesian_product(0..4)
                .cartesian_product(0..4)
                .flat_map(|((u, v), w)| {
                    let force = self.force(u, v, w) * self.masses[u][v][w];
                    [force.x, force.y, force.z]
                }),
        )
    }
}

impl PlainODE<ODE_DIM> for JellyODE {
    fn derivative(&self, state: &JellyState) -> na::SVector<f64, ODE_DIM> {
        na::SVector::from_iterator(
            state
                .y
                .iter()
                .skip(SPACE_DIM)
                .take(SPACE_DIM)
                .copied()
                .chain(self.accelerations().iter().copied()),
        )
    }
}
