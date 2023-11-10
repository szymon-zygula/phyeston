use nalgebra as na;

pub struct Inertia {
    matrix: na::Matrix3<f64>,
    inverse: na::Matrix3<f64>,
}

impl Inertia {
    pub fn new(matrix: na::Matrix3<f64>) -> Self {
        Self {
            inverse: matrix
                .clone()
                .try_inverse()
                .expect("Inertia tensor is not invertible"),
            matrix,
        }
    }

    pub fn matrix(&self) -> &na::Matrix3<f64> {
        &self.matrix
    }

    pub fn inverse_matrix(&self) -> &na::Matrix3<f64> {
        &self.inverse
    }
}
