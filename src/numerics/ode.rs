use super::Float;
use nalgebra as na;

/// Ordinary Differential Equation
pub trait ODE<F: Float, const DIM_OUT: usize> {
    fn derivative(&self) -> na::SVector<F, DIM_OUT>;

    fn t(&self) -> F;
    fn y(&self) -> na::SVector<F, DIM_OUT>;

    fn set_t(&mut self, t: F);
    fn set_y(&mut self, y: na::SVector<F, DIM_OUT>);
}
