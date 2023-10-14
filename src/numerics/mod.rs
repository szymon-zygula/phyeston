use nalgebra as na;

pub mod euler;
pub use euler::*;
pub mod ode;
pub use ode::*;

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
