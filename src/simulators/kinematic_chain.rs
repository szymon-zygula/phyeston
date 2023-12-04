use crate::numerics::{kinematics::flat_chain, Rect, Segment};
use crate::render::texture::Texture;
use image::Rgba;

use nalgebra as na;

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
                let state = self.system.forward_kinematics(&na::point![
                    (alpha_1 as f64).to_radians(),
                    (alpha_2 as f64).to_radians()
                ]);

                let segment_1_collision = Segment::new(self.origin, state.p_1 + self.origin.coords)
                    .collides_with_rect(rect);
                let segment_2_collision = Segment::new(
                    state.p_1 + self.origin.coords,
                    state.p_2 + self.origin.coords,
                )
                .collides_with_rect(rect);

                *obstruction |= segment_1_collision || segment_2_collision;
            }
        }
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
