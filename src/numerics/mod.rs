use nalgebra as na;

pub mod euler;
pub use euler::EulerODESolver;
pub mod ode;
pub use ode::ODE;
pub mod runge_kutta;
pub use runge_kutta::RungeKuttaIV;

pub trait Float:
    std::ops::AddAssign
    + std::ops::SubAssign
    + num_traits::Float
    + std::fmt::Debug
    + 'static
    + na::ComplexField
    + egui::emath::Numeric
{
}

impl<T> Float for T where
    T: std::ops::AddAssign
        + std::ops::SubAssign
        + num_traits::Float
        + std::fmt::Debug
        + 'static
        + na::ComplexField
        + egui::emath::Numeric
{
}

pub type FloatFn<F> = Box<dyn Fn(F) -> F>;
