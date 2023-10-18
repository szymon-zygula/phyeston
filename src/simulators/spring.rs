use crate::numerics::{ode::ODE, FloatFn};
use nalgebra as na;
use struct_iterable::Iterable;

pub type F = f64;

#[derive(Clone, Debug, Iterable)]
pub struct SpringState {
    pub t: F,

    pub position: F,
    pub velocity: F,
    pub acceleration: F,

    pub spring_force: F,
    pub damping_force: F,
    pub outer_force: F,
    pub total_force: F,

    pub equilibrium: F,
}

impl SpringState {
    pub fn iter(&self) -> Vec<(&'static str, F)> {
        Iterable::iter(self)
            .map(|(name, val)| (name, *val.downcast_ref::<F>().unwrap()))
            .collect()
    }
}

pub struct SpringODE {
    t: F,

    pub mass: F,
    pub equilibrium: FloatFn<F>,

    position: F,
    velocity: F,

    pub spring_constant: F,
    pub damping_factor: F,
    pub outer_force: FloatFn<F>,
}

impl SpringODE {
    pub fn new(
        mass: F,
        equilibrium: FloatFn<F>,
        position: F,
        velocity: F,
        spring_constant: F,
        damping_factor: F,
        outer_force: FloatFn<F>,
    ) -> Self {
        Self {
            t: 0.0,
            mass,
            equilibrium,
            position,
            velocity,
            spring_constant,
            damping_factor,
            outer_force,
        }
    }

    pub fn state(&self) -> SpringState {
        SpringState {
            t: self.t,

            position: self.position(),
            velocity: self.velocity(),
            acceleration: self.acceleration(),

            spring_force: self.spring_force(),
            damping_force: self.damping_force(),
            outer_force: self.outer_force(),
            total_force: self.total_force(),

            equilibrium: self.equilibrium(),
        }
    }

    pub fn total_force(&self) -> F {
        self.spring_force() + self.damping_force() + self.outer_force()
    }

    pub fn outer_force(&self) -> F {
        (self.outer_force)(self.t)
    }

    pub fn damping_force(&self) -> F {
        -self.damping_factor * self.velocity
    }

    pub fn spring_force(&self) -> F {
        self.spring_constant * (self.equilibrium() - self.position)
    }

    pub fn equilibrium(&self) -> F {
        (self.equilibrium)(self.t)
    }

    pub fn position(&self) -> F {
        self.position
    }

    pub fn velocity(&self) -> F {
        self.velocity
    }

    pub fn acceleration(&self) -> F {
        self.total_force() / self.mass
    }
}

impl ODE<F, 2> for SpringODE {
    fn derivative(&self) -> na::Vector2<F> {
        na::vector![self.velocity, self.acceleration()]
    }

    fn t(&self) -> F {
        self.t
    }

    fn y(&self) -> na::SVector<F, 2> {
        na::vector![self.position, self.velocity]
    }

    fn set_t(&mut self, t: F) {
        self.t = t;
    }

    fn set_y(&mut self, y: na::SVector<F, 2>) {
        self.position = y[0];
        self.velocity = y[1];
    }
}
