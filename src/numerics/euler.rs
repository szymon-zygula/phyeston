use super::{ode::ODE, Float};

pub struct EulerODESolver<F: Float, const DIM_OUT: usize, O: ODE<F, DIM_OUT>> {
    delta: F,
    ode: O,
}

impl<F: Float, const DIM_OUT: usize, O: ODE<F, DIM_OUT>> EulerODESolver<F, DIM_OUT, O> {
    pub fn new(step: F, ode: O) -> Self {
        Self { delta: step, ode }
    }

    pub fn step(&mut self) {
        let new_y = self.ode.y() + self.ode.derivative() * self.delta;
        let new_t = self.ode.t() + self.delta;

        self.ode.set_t(new_t);
        self.ode.set_y(new_y);
    }

    pub fn ode(&self) -> &O {
        &self.ode
    }

    pub fn ode_mut(&mut self) -> &mut O {
        &mut self.ode
    }

    pub fn take_ode(self) -> O {
        self.ode
    }

    pub fn step_mut(&mut self) -> &mut F {
        &mut self.delta
    }
}
