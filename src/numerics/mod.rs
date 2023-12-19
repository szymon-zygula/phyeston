use nalgebra as na;

pub mod bezier;
pub mod cylinder;
pub mod kinematics;
pub mod ode;
pub mod parametric;
pub mod rect;
pub mod rotations;
pub mod segment;

pub use ode::EulerODESolver;
pub use ode::RungeKuttaIV;
pub use ode::ODE;
pub use rect::Rect;
pub use segment::Segment;

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
