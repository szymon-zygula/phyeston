use nalgebra as na;

pub mod bezier;
pub mod ode;
pub mod rotations;
pub use ode::EulerODESolver;
pub use ode::RungeKuttaIV;
pub use ode::ODE;

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
