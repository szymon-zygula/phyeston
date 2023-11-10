use crate::{
    numerics::ode::{PlainODE, State},
    physics::inertia::Inertia,
};
use nalgebra as na;

pub struct SpinningTopODE {
    inertia: Inertia,
    torque: na::Vector3<f64>,
}

impl SpinningTopODE {
    const BOX_SIDE_LENGTH: f64 = 2.0;

    pub fn new(density: f64) -> Self {
        Self {
            inertia: Self::inertia(density),
            torque: na::Vector3::zeros(),
        }
    }

    fn inertia(density: f64) -> Inertia {
        Inertia::new(
            Self::BOX_SIDE_LENGTH.powi(5)
                * na::matrix![
                    2.0/3.0, -0.25, -0.25;
                    -0.25, 2.0 /3.0, -0.25;
                    -0.25, -0.25, 2.0/3.0;
                ],
        )
    }

    pub fn set_density(&mut self, density: f64) {
        self.inertia = Self::inertia(density);
    }
}

impl PlainODE<7> for SpinningTopODE {
    fn derivative(&self, state: &State<7>) -> na::SVector<f64, 7> {
        let angular_velocity = state.y.xyz();
        let quaternion = na::UnitQuaternion::new_normalize(na::Quaternion::new(
            state.y[3], state.y[4], state.y[5], state.y[6],
        ));

        let angular_velocity_derivative = self.inertia.inverse_matrix()
            * (self.torque + (self.inertia.matrix() * angular_velocity).cross(&angular_velocity));
        // Zrobić mnożenie kwaternionowe, normalizacja itd.
        let rotation_derivative = quaternion * angular_velocity * 0.5;

        na::vector![
            angular_velocity_derivative.x,
            angular_velocity_derivative.y,
            angular_velocity_derivative.z,
            todo!(),
            todo!(),
            todo!(),
            todo!(),
        ]
    }
}
