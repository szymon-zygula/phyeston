use nalgebra as na;

pub struct EulerODESolver<Float, const DIM_IN: usize, const DIM_OUT: usize> {
    current_y: na::SVector<Float, DIM_OUT>,
}

impl<Float, const DIM_IN: usize, const DIM_OUT: usize> EulerODESolver<Float, DIM_IN, DIM_OUT> {
    pub fn new() {}
}
