use crate::numerics::kinematics::flat_chain;
use crate::render::texture::Texture;
use image::Rgba;

use nalgebra as na;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub p_1: na::Point2<f64>,
    pub p_2: na::Point2<f64>,
}

pub const CONFIG_OBSTRUCTION_SIZE: usize = 360;

pub struct ConfigObstuction {
    obstructed: [[bool; CONFIG_OBSTRUCTION_SIZE]; CONFIG_OBSTRUCTION_SIZE],
    system: flat_chain::System,
    origin: na::Point2<f64>,
}

impl ConfigObstuction {
    pub fn new(system: flat_chain::System, origin: na::Point2<f64>) -> Self {
        Self {
            obstructed: [[false; CONFIG_OBSTRUCTION_SIZE]; CONFIG_OBSTRUCTION_SIZE],
            system,
            origin,
        }
    }

    pub fn add_rect(&mut self, rect: &Rect) {
        for (alpha_1, subarray) in self.obstructed.iter_mut().enumerate() {
            for (alpha_2, obstruction) in subarray.iter_mut().enumerate() {
                let state = self
                    .system
                    .forward_kinematics(&na::point![alpha_1 as f64, alpha_2 as f64]);

                *obstruction = Self::collides_with_rect(&self.origin, &state.p_1, rect)
                    || Self::collides_with_rect(&state.p_1, &state.p_2, rect);
            }
        }
    }

    fn collides_with_rect(p_1: &na::Point2<f64>, p_2: &na::Point2<f64>, rect: &Rect) -> bool {
        p_1.x < rect.p_1.x && rect.p_1.x < p_2.x
            || p_1.x < rect.p_2.x && rect.p_2.x < p_2.x
            || p_1.y < rect.p_1.y && rect.p_1.y < p_2.y
            || p_1.y < rect.p_2.y && rect.p_2.y < p_2.y
            || p_2.x < rect.p_1.x && rect.p_1.x < p_1.x
            || p_2.x < rect.p_2.x && rect.p_2.x < p_1.x
            || p_2.y < rect.p_1.y && rect.p_1.y < p_1.y
            || p_2.y < rect.p_2.y && rect.p_2.y < p_1.y
    }

    pub fn texture(&self) -> Texture {
        let mut texture = Texture::new_rgb(
            CONFIG_OBSTRUCTION_SIZE as u32,
            CONFIG_OBSTRUCTION_SIZE as u32,
        );

        for (alpha_1, subarray) in self.obstructed.iter().enumerate() {
            for (alpha_2, &obstructed) in subarray.iter().enumerate() {
                texture.put(
                    alpha_1 as u32,
                    alpha_2 as u32,
                    Rgba([0, if obstructed { 255 } else { 0 }, 0, 255]),
                );
            }
        }

        texture
    }
}
