use super::{PlainODE, Solver, SolverWithDelta, State};

pub struct RungeKuttaIV<const DIM_OUT: usize, O: PlainODE<DIM_OUT>> {
    pub delta: f64,
    pub ode: O,
}

impl<const DIM_OUT: usize, O: PlainODE<DIM_OUT>> RungeKuttaIV<DIM_OUT, O> {
    pub fn new(step: f64, ode: O) -> Self {
        Self { delta: step, ode }
    }
}

impl<const DIM_OUT: usize, O: PlainODE<DIM_OUT>> Solver<DIM_OUT, O> for RungeKuttaIV<DIM_OUT, O> {
    fn step(&self, state: &State<DIM_OUT>) -> State<DIM_OUT> {
        let h = self.delta;
        let t = state.t;
        let y = &state.y;

        let k1 = self.ode.derivative(state);

        let k2 = self.ode.derivative(&State {
            t: t + h * 0.5,
            y: y + k1 * h * 0.5,
        });

        let k3 = self.ode.derivative(&State {
            t: t + h * 0.5,
            y: y + k2 * h * 0.5,
        });

        let k4 = self.ode.derivative(&State {
            t: t + h * 0.5,
            y: y + k3 * h,
        });

        State {
            t: t + h,
            y: y + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * h / 6.0,
        }
    }

    fn replace_ode(&mut self, mut ode: O) -> O {
        std::mem::swap(&mut self.ode, &mut ode);
        ode
    }

    fn take_ode(self) -> O {
        self.ode
    }

    fn ode_mut(&mut self) -> &mut O {
        &mut self.ode
    }

    fn ode(&self) -> &O {
        &self.ode
    }
}

impl<const DIM_OUT: usize, O: PlainODE<DIM_OUT>> SolverWithDelta<DIM_OUT, O>
    for RungeKuttaIV<DIM_OUT, O>
{
    fn delta_mut(&mut self) -> &mut f64 {
        &mut self.delta
    }

    fn delta(&self) -> f64 {
        self.delta
    }
}
