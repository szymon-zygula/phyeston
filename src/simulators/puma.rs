use crate::numerics::{angle::Angle, rotations::*};
use nalgebra as na;

#[derive(Default, Clone, Copy)]
pub struct ConfigState {
    pub a1: Angle,
    pub a2: Angle,
    pub a3: Angle,
    pub a4: Angle,
    pub a5: Angle,
    pub q2: f64,
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            q2: 1.0,
            ..Default::default()
        }
    }

    pub fn forward_kinematics(&self, params: &Params) -> CylindersTransforms {
        let f01 = rotate_z(self.a1.rad());
        let f11 = na::Translation3::new(0.0, 0.0, params.l1).to_homogeneous();
        let f11_half = na::Translation3::new(0.0, 0.0, params.l1 * 0.5).to_homogeneous();
        let f12 = rotate_y(self.a2.rad());
        let f22 = na::Translation3::new(self.q2, 0.0, 0.0).to_homogeneous();
        let f22_half = na::Translation3::new(self.q2 * 0.5, 0.0, 0.0).to_homogeneous();
        let f23 = rotate_y(self.a3.rad());
        let f33 = na::Translation3::new(0.0, 0.0, -params.l3).to_homogeneous();
        let f33_half = na::Translation3::new(0.0, 0.0, -params.l3 * 0.5).to_homogeneous();
        let f34 = rotate_z(self.a4.rad());
        let f44 = na::Translation3::new(params.l4, 0.0, 0.0).to_homogeneous();
        let f44_half = na::Translation3::new(params.l4 * 0.5, 0.0, 0.0).to_homogeneous();
        let f45 = rotate_x(self.a5.rad());

        let thin = na::Scale3::new(0.1, 0.1, 1.0).to_homogeneous();
        let scale1 = na::Scale3::new(1.0, 1.0, 0.5 * params.l1).to_homogeneous();
        let scale2 = na::Scale3::new(1.0, 1.0, 0.5 * self.q2).to_homogeneous();
        let scale3 = na::Scale3::new(1.0, 1.0, 0.5 * params.l3).to_homogeneous();
        let scale4 = na::Scale3::new(1.0, 1.0, 0.5 * params.l4).to_homogeneous();

        let rot2 = rotate_y(std::f64::consts::FRAC_PI_2);
        let rot3 = rotate_y(std::f64::consts::PI);
        let rot4 = rotate_y(std::f64::consts::FRAC_PI_2);

        CylindersTransforms {
            bone_transforms: [
                f01 * f11_half * thin * scale1,
                f01 * f11 * f12 * f22_half * rot2 * thin * scale2,
                f01 * f11 * f12 * f22 * f23 * f33_half * rot3 * thin * scale3,
                f01 * f11 * f12 * f22 * f23 * f33 * f34 * f44_half * rot4 * thin * scale4,
                f01 * f11 * f12 * f22 * f23 * f33 * f34 * f44 * f45,
            ],
            joint_transforms: [
                na::Matrix4::identity(),
                na::Matrix4::identity(),
                na::Matrix4::identity(),
                na::Matrix4::identity(),
                na::Matrix4::identity(),
                na::Matrix4::identity(),
            ],
        }
    }

    pub fn next_config(&self, next_position: &SceneState) {
        let solution = next_position.inverse_kinematics();
        self.closest_config(&solution)
    }

    pub fn closest_config(&self, solution: &InverseSolution) {}

    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            a1: self.a1.lerp(other.a1, t),
            a2: self.a2.lerp(other.a2, t),
            a3: self.a3.lerp(other.a3, t),
            a4: self.a4.lerp(other.a4, t),
            a5: self.a5.lerp(other.a5, t),
            q2: self.q2 * (1.0 - t) + other.q2 * t,
        }
    }
}

pub enum InverseSolution {}

#[derive(Clone)]
pub struct CylindersTransforms {
    pub bone_transforms: [na::Matrix4<f64>; 5],
    pub joint_transforms: [na::Matrix4<f64>; 6],
}

pub struct SceneState {
    pub position: na::Point3<f64>,
    pub direction: Quaternion,
}

impl SceneState {
    pub fn inverse_kinematics(&self) -> InverseSolution {
        todo!()
    }
}

pub struct Params {
    pub l1: f64,
    pub l3: f64,
    pub l4: f64,
}
