use crate::{
    numerics::ode::{PlainODE, State},
    physics::inertia::Inertia,
};
use nalgebra as na;

pub struct SpinningTopODE {
    inertia: Inertia,
    side_length: f64,
    density: f64,
    pub gravity: na::Vector3<f64>,
    pub enable_gravity: bool,
}

impl SpinningTopODE {
    pub fn new(density: f64, side_length: f64) -> Self {
        let mut me = Self {
            inertia: Inertia::unit(),
            gravity: na::Vector3::new(0.0, -100.0, 0.0),
            enable_gravity: true,
            density,
            side_length,
        };

        me.calc_inertia();

        me
    }

    fn calc_inertia(&mut self) {
        self.inertia = Inertia::new(
            self.density
                * self.side_length.powi(5)
                * na::matrix![
                    2.0/3.0, -0.25, -0.25;
                    -0.25, 2.0 /3.0, -0.25;
                    -0.25, -0.25, 2.0/3.0;
                ],
        );
    }

    pub fn torque(&self, rotation: &na::UnitQuaternion<f64>) -> na::Vector3<f64> {
        let natural_center = 0.5 * self.side_length() * na::vector![1.0, 1.0, 1.0];

        rotation.inverse().transform_vector(
            &rotation
                .transform_vector(&natural_center)
                .cross(&self.weight()),
        )
    }

    pub fn weight(&self) -> na::Vector3<f64> {
        self.gravity() * self.mass()
    }

    pub fn mass(&self) -> f64 {
        self.density * self.side_length.powi(3)
    }

    pub fn side_length(&self) -> f64 {
        self.side_length
    }

    pub fn set_side_length(&mut self, side_length: f64) {
        self.side_length = side_length;
        self.calc_inertia()
    }

    pub fn density(&self) -> f64 {
        self.density
    }

    pub fn set_density(&mut self, density: f64) {
        self.density = density;
        self.calc_inertia();
    }

    pub fn gravity(&self) -> na::Vector3<f64> {
        if self.enable_gravity {
            self.gravity
        } else {
            na::Vector3::zeros()
        }
    }
}

impl PlainODE<7> for SpinningTopODE {
    fn derivative(&self, state: &State<7>) -> na::SVector<f64, 7> {
        let angular_velocity = state.y.xyz();
        let rotation = na::UnitQuaternion::new_normalize(na::Quaternion::new(
            state.y[3], state.y[4], state.y[5], state.y[6],
        ));

        let angular_velocity_derivative = self.inertia.inverse_matrix()
            * (self.torque(&rotation)
                + (self.inertia.matrix() * angular_velocity).cross(&angular_velocity));
        let angular_velocity_quaternion = na::Quaternion::new(
            0.0,
            angular_velocity.x,
            angular_velocity.y,
            angular_velocity.z,
        );

        let rotation_derivative = rotation.quaternion() * angular_velocity_quaternion * 0.5;

        na::vector![
            angular_velocity_derivative.x,
            angular_velocity_derivative.y,
            angular_velocity_derivative.z,
            rotation_derivative.w,
            rotation_derivative.i,
            rotation_derivative.j,
            rotation_derivative.k,
        ]
    }
}
