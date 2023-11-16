pub mod euler;
pub use euler::EulerODESolver;
pub mod runge_kutta;
pub use runge_kutta::RungeKuttaIV;

use super::Float;
use nalgebra as na;

#[derive(Debug)]
pub struct State<const DIM_OUT: usize> {
    pub t: f64,
    pub y: na::SVector<f64, DIM_OUT>,
}

/// Ordinary Differential Equation
pub trait PlainODE<const DIM_OUT: usize> {
    fn derivative(&self, state: &State<DIM_OUT>) -> na::SVector<f64, DIM_OUT>;
}

/// Ordinary Differential Equation which owns its `t` and `y`.
// TODO: Remove and leave just `PlainODE`
pub trait ODE<F: Float, const DIM_OUT: usize> {
    fn derivative(&self) -> na::SVector<F, DIM_OUT>;

    fn t(&self) -> F;
    fn y(&self) -> na::SVector<F, DIM_OUT>;

    fn set_t(&mut self, t: F);
    fn set_y(&mut self, y: na::SVector<F, DIM_OUT>);
}

/// Ordinary Differential Equation Solver
pub trait Solver<const DIM_OUT: usize, O: PlainODE<DIM_OUT>> {
    fn step(&self, state: &State<DIM_OUT>) -> State<DIM_OUT>;
    fn replace_ode(&mut self, ode: O) -> O;
    fn take_ode(self) -> O;
    fn ode_mut(&mut self) -> &mut O;
    fn ode(&self) -> &O;
}
