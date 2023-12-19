use nalgebra as na;

pub trait ParametricForm<const IN_DIM: usize, const OUT_DIM: usize> {
    fn bounds(&self) -> na::SVector<(f64, f64), IN_DIM>;
    fn wrapped(&self, dim: usize) -> bool;
    fn value(&self, vec: &na::SVector<f64, IN_DIM>) -> na::Point<f64, OUT_DIM>;
    fn normal(&self, vec: &na::Vector2<f64>) -> na::SVector<f64, OUT_DIM>;
}
