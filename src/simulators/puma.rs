use crate::numerics::rotations::*;
use nalgebra as na;

pub struct ConfigState {
    pub a1: f64,
    pub a2: f64,
    pub a3: f64,
    pub a4: f64,
    pub a5: f64,
    pub q2: f64,
}

impl ConfigState {
    pub fn forward_kinematics(&self) -> CylinderTransform {
        todo!()
    }

    pub fn next_config(&self, next_position: &SceneState) {
        let solution = next_position.inverse_kinematics();
        self.closest_config(&solution)
    }

    pub fn closest_config(&self, solution: &InverseSolution) {}
}

pub enum InverseSolution {}

pub struct CylinderTransform {}

pub struct SceneState {
    pub position: na::Point3<f64>,
    pub direction: Quaternion,
}

impl SceneState {
    pub fn inverse_kinematics(&self) -> InverseSolution {
        todo!()
    }
}

pub struct Params {
    pub l1: f64,
    pub l3: f64,
    pub l4: f64,
}
