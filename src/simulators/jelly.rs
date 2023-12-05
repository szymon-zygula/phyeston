use crate::numerics::{
    bezier,
    ode::{PlainODE, State},
};
use itertools::Itertools;
use nalgebra as na;
use std::array;
use std::cell::RefCell;
use std::rc::Rc;

pub const POINT_COUNT: usize = 64;
pub const SPACE_DIM: usize = POINT_COUNT * 3;
pub const ODE_DIM: usize = SPACE_DIM * 2;

pub type JellyState = State<ODE_DIM>;

pub struct ControlFrameTransform {
    pub translation: na::Vector3<f64>,
    pub rotation: na::Quaternion<f64>,
}

impl ControlFrameTransform {
    pub fn new() -> Self {
        Self {
            translation: na::Vector3::zeros(),
            rotation: na::Quaternion::new(1.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn compose(&self) -> na::Matrix4<f64> {
        na::Translation3::from(self.translation).to_homogeneous()
            * na::Rotation3::from(na::UnitQuaternion::new_normalize(self.rotation)).to_homogeneous()
    }
}
pub struct JellyODE {
    pub masses: [[[f64; 4]; 4]; 4],
    pub corner_spring_constant: f64,
    pub inner_spring_constant: f64,
    pub damping_factor: f64,
    control_frame: Rc<RefCell<ControlFrameTransform>>,
}

impl JellyODE {
    pub fn new(control_frame: Rc<RefCell<ControlFrameTransform>>) -> Self {
        let masses = array::from_fn(|_| array::from_fn(|_| array::from_fn(|_| 1.0)));

        Self {
            masses,
            corner_spring_constant: 10.0,
            inner_spring_constant: 3.0,
            damping_factor: 1.0,
            control_frame,
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

    /// Force acting on point `p_1`
    fn spring_force(
        p_0: &na::Point3<f64>,
        p_1: &na::Point3<f64>,
        length: f64,
        spring_constant: f64,
    ) -> na::Vector3<f64> {
        let diff = p_0 - p_1;
        let dist = (diff).norm();

        if dist == 0.0 {
            na::Vector3::zeros()
        } else {
            diff / dist * spring_constant * (dist - length)
        }
    }

    fn corner_coord(i: usize) -> f64 {
        if i == 0 {
            -1.0
        } else {
            1.0
        }
    }

    fn corner_point(transform: &na::Matrix4<f64>, u: usize, v: usize, w: usize) -> na::Point3<f64> {
        na::Point3::from_homogeneous(
            transform
                * na::point![
                    Self::corner_coord(u),
                    Self::corner_coord(v),
                    Self::corner_coord(w),
                ]
                .to_homogeneous(),
        )
        .unwrap_or(na::Point3::origin())
    }

    fn corner_force(
        &self,
        frame_transform: &na::Matrix4<f64>,
        state: &JellyState,
        u: usize,
        v: usize,
        w: usize,
    ) -> na::Vector3<f64> {
        if (u != 3 && u != 0) || (v != 3 && v != 0) || (w != 3 && w != 0) {
            na::Vector3::zeros()
        } else {
            let corner_point = Self::corner_point(frame_transform, u, v, w);
            let idx = (w + v * 4 + u * 16) * 3;
            Self::spring_force(
                &corner_point,
                &na::point![state.y[idx], state.y[idx + 1], state.y[idx + 2]],
                0.0,
                self.corner_spring_constant,
            )
        }
    }

    fn coord_neigh_range(i: i64) -> &'static [i64] {
        if i == 0 {
            &[0, 1]
        } else if i == 3 {
            &[-1, 0]
        } else {
            &[-1, 0, 1]
        }
    }

    fn inner_force(&self, state: &JellyState, u: usize, v: usize, w: usize) -> na::Vector3<f64> {
        let u = u as i64;
        let v = v as i64;
        let w = w as i64;
        let mut force_accumulator = na::Vector3::zeros();

        for &du in Self::coord_neigh_range(u) {
            for &dv in Self::coord_neigh_range(v) {
                for &dw in Self::coord_neigh_range(w) {
                    if (du == 0 && dv == 0 && dw == 0) || (du != 0 && dv != 0 && dw != 0) {
                        continue;
                    }

                    let idx = (w + v * 4 + u * 16) as usize * 3;
                    let idx_other = ((w + dw) + (v + dv) * 4 + (u + du) * 16) as usize * 3;
                    let position = na::point![state.y[idx], state.y[idx + 1], state.y[idx + 2]];
                    let other_position = na::point![
                        state.y[idx_other],
                        state.y[idx_other + 1],
                        state.y[idx_other + 2]
                    ];

                    let diagonal_spring = ((du + dv + dw) % 2).abs() == 0;

                    force_accumulator += Self::spring_force(
                        &other_position,
                        &position,
                        2.0 / 3.0
                            * if diagonal_spring {
                                std::f64::consts::SQRT_2
                            } else {
                                1.0
                            },
                        self.inner_spring_constant,
                    );
                }
            }
        }

        force_accumulator
    }

    fn damping_force(&self, state: &JellyState, u: usize, v: usize, w: usize) -> na::Vector3<f64> {
        let idx = (w + v * 4 + u * 16) * 3 + SPACE_DIM;
        let velocity = na::vector![state.y[idx], state.y[idx + 1], state.y[idx + 2]];
        -velocity * self.damping_factor
    }

    fn force(
        &self,
        frame_transform: &na::Matrix4<f64>,
        state: &JellyState,
        u: usize,
        v: usize,
        w: usize,
    ) -> na::Vector3<f64> {
        self.corner_force(frame_transform, state, u, v, w)
            + self.inner_force(state, u, v, w)
            + self.damping_force(state, u, v, w)
    }

    fn accelerations(
        &self,
        frame_transform: &na::Matrix4<f64>,
        state: &JellyState,
    ) -> na::SVector<f64, SPACE_DIM> {
        na::SVector::from_iterator(
            (0..4)
                .cartesian_product(0..4)
                .cartesian_product(0..4)
                .flat_map(|((u, v), w)| {
                    let force = self.force(frame_transform, state, u, v, w) * self.masses[u][v][w];
                    [force.x, force.y, force.z]
                }),
        )
    }
}

impl PlainODE<ODE_DIM> for JellyODE {
    fn derivative(&self, state: &JellyState) -> na::SVector<f64, ODE_DIM> {
        let frame_transform = self.control_frame.borrow().compose();
        na::SVector::from_iterator(
            state
                .y
                .iter()
                .skip(SPACE_DIM)
                .take(SPACE_DIM)
                .copied()
                .chain(self.accelerations(&frame_transform, state).iter().copied()),
        )
    }
}
