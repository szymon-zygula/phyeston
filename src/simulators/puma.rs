use crate::numerics::{angle::Angle, rotations::*};
use nalgebra as na;
use std::f64::consts::PI;

#[derive(Debug, Default, Clone, Copy)]
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

        let f02 = f01 * f11 * f12;
        let f03 = f02 * f22 * f23;
        let f04 = f03 * f33 * f34;
        let f05 = f04 * f44 * f45;

        CylindersTransforms {
            bone_transforms: [
                f01 * f11_half * thin * scale1,
                f02 * f22_half * rot2 * thin * scale2,
                f03 * f33_half * rot3 * thin * scale3,
                f04 * f44_half * rot4 * thin * scale4,
                f05,
            ],
            joint_transforms: [
                na::Scale3::new(2.0, 2.0, 2.0).to_homogeneous(),
                f01 * f11 * rotate_x(std::f64::consts::FRAC_PI_2),
                f02 * f22 * rotate_x(std::f64::consts::FRAC_PI_2),
                f03 * f33,
            ]
            .map(|m| m * na::Scale3::new(0.2, 0.2, 0.2).to_homogeneous()),
        }
    }

    pub fn next_config(&self, next_position: &SceneState, params: &Params) -> Self {
        next_position.inverse_kinematics(&self, params)
    }

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

#[derive(Clone)]
pub struct CylindersTransforms {
    pub bone_transforms: [na::Matrix4<f64>; 5],
    pub joint_transforms: [na::Matrix4<f64>; 4],
}

pub struct SceneState {
    pub position: na::Point3<f64>,
    pub rotation: Quaternion,
}

impl SceneState {
    pub fn new(position: na::Point3<f64>, rotation: Quaternion) -> Self {
        Self { position, rotation }
    }

    pub fn interpolate(&self, other: &Self, t: f64) -> Self {
        Self::new(
            self.position.lerp(&other.position, t),
            self.rotation.slerp(&other.rotation, t),
        )
    }

    pub fn inverse_kinematics(&self, guide: &ConfigState, params: &Params) -> ConfigState {
        // Effector is at p4, its axes are i5, j5 and k5
        let p4 = self.position;
        let p4 = na::vector![p4.x, p4.y, p4.z, 1.0];
        let d4x = (self.rotation.to_homogeneous() * na::vector![1.0, 0.0, 0.0, 0.0]).normalize();
        let i = d4x.x;
        let j = d4x.y;
        let k = d4x.z;

        let p3 = p4 - params.l4 * d4x;

        let a1 = if p3.x != 0.0 || p3.y != 0.0 {
            let a1_abs = Angle::from_rad(f64::atan2(p3.y, p3.x).abs());
            let c1 = a1_abs.cos();

            let a1_mod_pi = if c1 * p3.x > 0.0 {
                if p3.y > 0.0 {
                    a1_abs
                } else {
                    -a1_abs
                }
            } else {
                if p3.y > 0.0 {
                    -a1_abs
                } else {
                    a1_abs
                }
            };

            guide.a1.closest(a1_mod_pi, a1_mod_pi + Angle::pi_rad())
        } else {
            guide.a1
        };

        let s1 = a1.sin();
        let c1 = a1.cos();

        let icjs = i * c1 + j * s1;

        // a2 + a3
        let a23 = if k == 0.0 && icjs == 0.0 {
            guide.a2 + guide.a3
        } else {
            let a23_mod_pi = Angle::from_rad(f64::atan2(k, -icjs));
            (guide.a2 + guide.a3).closest(a23_mod_pi, a23_mod_pi + Angle::pi_rad())
        };

        let s23 = a23.sin();
        let c23 = a23.cos();

        let y_a2 = params.l1 - params.l3 * c23 - p3.z;
        let x_a2 = if c1.abs() < s1.abs() {
            p3.y / s1
        } else {
            p3.x / c1
        } + params.l3 * s23;

        let a2 = if x_a2 == 0.0 && y_a2 == 0.0 {
            guide.a2
        } else {
            Angle::from_rad(f64::atan2(y_a2, x_a2))
        };

        let s2 = a2.sin();
        let c2 = a2.cos();

        let a3 = a23 - a2;

        let q2 = if s2.abs() > c2.abs() {
            (params.l1 - params.l3 * c23 - p3.z) / s2
        } else {
            (if s1.abs() > c1.abs() {
                p3.y / s1
            } else {
                p3.x / c1
            } + params.l3 * s23)
                / c2
        };

        let d3x = (rotate_z(a1.rad())
            * rotate_y(a2.rad())
            * rotate_y(a3.rad())
            * na::vector![1.0, 0.0, 0.0, 0.0])
        .normalize();

        let d3z = (rotate_z(a1.rad())
            * rotate_y(a2.rad())
            * rotate_y(a3.rad())
            * na::vector![0.0, 0.0, 1.0, 0.0])
        .normalize();

        let c4 = na::Vector3::dot(&d3x.xyz(), &d4x.xyz()).clamp(-1.0, 1.0);
        let a4_abs = Angle::from_rad(c4.acos());

        let a4 = if na::Vector3::cross(&d3x.xyz(), &d4x.xyz()).dot(&d3z.xyz()) > 0.0 {
            a4_abs
        } else {
            -a4_abs
        };

        let d4z = d3z; // Rotation 4 is around Z axis
        let d5z = (self.rotation.to_homogeneous() * na::vector![0.0, 0.0, 1.0, 0.0]).normalize();

        let c5 = na::Vector3::dot(&d4z.xyz(), &d5z.xyz()).clamp(-1.0, 1.0);
        let a5_abs = Angle::from_rad(c5.acos());

        let a5 = if na::Vector3::cross(&d4z.xyz(), &d5z.xyz()).dot(&d4x.xyz()) > 0.0 {
            a5_abs
        } else {
            -a5_abs
        };

        ConfigState {
            a1,
            a2,
            a3,
            a4,
            a5,
            q2,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Params {
    pub l1: f64,
    pub l3: f64,
    pub l4: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            l1: 3.0,
            l3: 3.0,
            l4: 3.0,
        }
    }
}
