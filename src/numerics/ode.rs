use nalgebra as na;

/// Ordinary Differential Equation
pub trait ODE<Float, const DIM_IN: usize, const DIM_OUT: usize> {
    fn derivative(t: Float, y: na::SVector<Float, DIM_IN>) -> na::SVector<Float, DIM_OUT>;
}
