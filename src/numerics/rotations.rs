use nalgebra as na;
use std::f64::consts::FRAC_PI_2;

#[derive(Clone, Copy, Debug)]
pub struct Quaternion(pub na::Vector4<f64>);

impl Quaternion {
    pub fn to_euler(&self) -> EulerAngles {
        let w = self.0[0];
        let x = self.0[1];
        let y = self.0[2];
        let z = self.0[3];

        let x_angle = f64::atan2(2.0 * (w * x + y * z), 1.0 - 2.0 * (x * x + y * y)).to_degrees();
        let y_angle = (-FRAC_PI_2
            + 2.0
                * f64::atan2(
                    (1.0 + 2.0 * (w * y - x * z)).max(0.0).sqrt(),
                    (1.0 - 2.0 * (w * y - x * z)).max(0.0).sqrt(),
                ))
        .to_degrees();
        let z_angle = f64::atan2(2.0 * (w * z + x * y), 1.0 - 2.0 * (y * y + z * z)).to_degrees();

        EulerAngles(na::vector![x_angle, y_angle, z_angle]).normalize()
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&c| c == 0.0)
    }

    pub fn lerp(&self, other: &Quaternion, t: f64) -> Quaternion {
        if other.is_zero() && self.is_zero() {
            return Quaternion(na::vector![1.0, 0.0, 0.0, 0.0]);
        }

        let new = Quaternion(self.0 * (1.0 - t) + other.0 * t);
        if new.is_zero() {
            let new_t = t + if t == 1.0 { -1.0 } else { 1.0 } * 10.0 * f64::EPSILON;

            self.lerp(other, new_t)
        } else {
            new.normalize()
        }
    }

    pub fn slerp(&self, other: &Quaternion, t: f64) -> Quaternion {
        let dot = self.0.dot(&other.0).clamp(-1.0, 1.0);
        let other = if dot < 0.0 { -*other } else { *other };

        let omega = dot.acos();

        if omega.sin().abs() <= 10.0 * f64::EPSILON {
            self.lerp(&other, t)
        } else {
            Quaternion(
                (((1.0 - t) * omega).sin() * self.0 + (t * omega).sin() * other.0) / omega.sin(),
            )
            .normalize()
        }
    }

    pub fn conjugate(&self) -> Self {
        Self(na::vector![self.0[0], -self.0[1], -self.0[2], -self.0[3]])
    }

    pub fn to_homogeneous(&self) -> na::Matrix4<f64> {
        let x = (*self * Quaternion(na::vector![0.0, 1.0, 0.0, 0.0]) * self.conjugate()).0;
        let y = (*self * Quaternion(na::vector![0.0, 0.0, 1.0, 0.0]) * self.conjugate()).0;
        let z = (*self * Quaternion(na::vector![0.0, 0.0, 0.0, 1.0]) * self.conjugate()).0;

        na::matrix![
            x[1], y[1], z[1], 0.0;
            x[2], y[2], z[2], 0.0;
            x[3], y[3], z[3], 0.0;
            0.0, 0.0, 0.0, 1.0;
        ]
    }

    pub fn normalize(&self) -> Self {
        Quaternion(self.0.normalize())
    }
}

impl std::ops::Mul for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let w1 = self.0[0];
        let x1 = self.0[1];
        let y1 = self.0[2];
        let z1 = self.0[3];

        let w2 = rhs.0[0];
        let x2 = rhs.0[1];
        let y2 = rhs.0[2];
        let z2 = rhs.0[3];

        Quaternion(na::vector![
            w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
            w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
            w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
            w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2
        ])
    }
}

impl std::ops::Neg for Quaternion {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Quaternion(-self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EulerAngles(pub na::Vector3<f64>);

impl EulerAngles {
    pub fn to_quaternion(&self) -> Quaternion {
        let psi_2 = self.0[2].to_radians() * 0.5;
        let theta_2 = self.0[1].to_radians() * 0.5;
        let phi_2 = self.0[0].to_radians() * 0.5;

        Quaternion(na::vector![psi_2.cos(), 0.0, 0.0, psi_2.sin()])
            * Quaternion(na::vector![theta_2.cos(), 0.0, theta_2.sin(), 0.0])
            * Quaternion(na::vector![phi_2.cos(), phi_2.sin(), 0.0, 0.0])
    }

    pub fn lerp(&self, other: &EulerAngles, t: f64) -> EulerAngles {
        let mut me = *self;
        let mut other = *other;
        if (other.0.x - me.0.x).abs() > 180.0 {
            if other.0.x > me.0.x {
                other.0.x -= 360.0;
            } else {
                me.0.x -= 360.0;
            }
        }

        if (other.0.y - me.0.y).abs() > 180.0 {
            if other.0.y > me.0.y {
                other.0.y -= 360.0;
            } else {
                me.0.y -= 360.0;
            }
        }

        if (other.0.z - me.0.z).abs() > 180.0 {
            if other.0.z > me.0.z {
                other.0.z -= 360.0;
            } else {
                me.0.z -= 360.0;
            }
        }

        EulerAngles(me.0 * (1.0 - t) + other.0 * t)
    }

    pub fn to_homogeneous(&self) -> na::Matrix4<f64> {
        rotate_z(self.0[2].to_radians())
            * rotate_y(self.0[1].to_radians())
            * rotate_x(self.0[0].to_radians())
    }

    pub fn normalize(&self) -> EulerAngles {
        EulerAngles(self.0.map(|c| c.rem_euclid(360.0)))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rotation {
    Quaternion(Quaternion),
    EulerAngles(EulerAngles),
}

impl Rotation {
    pub fn to_quaternion(&self) -> Quaternion {
        match self {
            Rotation::Quaternion(q) => *q,
            Rotation::EulerAngles(e) => e.to_quaternion(),
        }
    }

    pub fn to_euler_angles(&self) -> EulerAngles {
        match self {
            Rotation::Quaternion(q) => q.to_euler(),
            Rotation::EulerAngles(e) => *e,
        }
    }

    pub fn normalize(&self) -> Self {
        match self {
            Rotation::Quaternion(quaternion) => Self::Quaternion(quaternion.normalize()),
            Rotation::EulerAngles(euler) => Self::EulerAngles(euler.normalize()),
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::Quaternion(Quaternion(na::Vector4::new(1.0, 0.0, 0.0, 0.0)))
    }
}

pub fn rotate_x(angle: f64) -> na::Matrix4<f64> {
    let mut rot_x = na::Matrix4::zeros();

    rot_x[(0, 0)] = 1.0;
    rot_x[(3, 3)] = 1.0;

    rot_x[(1, 1)] = angle.cos();
    rot_x[(1, 2)] = -angle.sin();
    rot_x[(2, 1)] = angle.sin();
    rot_x[(2, 2)] = angle.cos();

    rot_x
}

pub fn rotate_y(angle: f64) -> na::Matrix4<f64> {
    let mut rot_y = na::Matrix4::zeros();

    rot_y[(1, 1)] = 1.0;
    rot_y[(3, 3)] = 1.0;

    rot_y[(0, 0)] = angle.cos();
    rot_y[(0, 2)] = angle.sin();
    rot_y[(2, 0)] = -angle.sin();
    rot_y[(2, 2)] = angle.cos();

    rot_y
}

pub fn rotate_z(angle: f64) -> na::Matrix4<f64> {
    let mut rot_z = na::Matrix4::zeros();

    rot_z[(2, 2)] = 1.0;
    rot_z[(3, 3)] = 1.0;

    rot_z[(0, 0)] = angle.cos();
    rot_z[(0, 1)] = -angle.sin();
    rot_z[(1, 0)] = angle.sin();
    rot_z[(1, 1)] = angle.cos();

    rot_z
}
