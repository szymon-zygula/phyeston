use crate::numerics::{kinematics::flat_chain, Rect, Segment};
use crate::render::texture::Texture;
use image::Rgba;
use itertools::Itertools;
use std::collections::VecDeque;

use nalgebra as na;

pub const CONFIG_SIZE: usize = 360;
pub const CONFIG_RANGE: std::ops::Range<i64> = 0..(CONFIG_SIZE as i64);

#[derive(Clone, Copy)]
struct BFSTrove {
    previous: Option<(usize, usize)>,
    distance: usize,
}

#[derive(Clone, Copy)]
struct IndexedBFSTrove {
    trove: BFSTrove,
    alpha_1: usize,
    alpha_2: usize,
}

pub struct BFSMap(Vec<[Option<BFSTrove>; CONFIG_SIZE]>);

impl BFSMap {
    pub fn empty() -> Self {
        Self(vec![[None; CONFIG_SIZE]; CONFIG_SIZE])
    }

    pub fn from_obstructions(start: &Option<na::Point2<f64>>, config: &ConfigObstuction) -> Self {
        let mut troves: Vec<[Option<BFSTrove>; CONFIG_SIZE]> =
            vec![[None; CONFIG_SIZE]; CONFIG_SIZE];

        let Some(start) = start else {
            return Self(troves);
        };

        let mut queue = VecDeque::from([IndexedBFSTrove {
            alpha_1: start.x.to_degrees().rem_euclid(360.0).floor() as usize,
            alpha_2: start.y.to_degrees().rem_euclid(360.0).floor() as usize,
            trove: BFSTrove {
                previous: None,
                distance: 0,
            },
        }]);

        while let Some(node) = queue.pop_front() {
            if let Some(existing_trove) = troves[node.alpha_1][node.alpha_2] {
                if existing_trove.distance <= node.trove.distance {
                    continue;
                }
            }

            troves[node.alpha_1][node.alpha_2] = Some(node.trove);

            for (d_1, d_2) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
                let new_alpha_1 =
                    (node.alpha_1 as i64 + d_1).rem_euclid(CONFIG_SIZE as i64) as usize;
                let new_alpha_2 =
                    (node.alpha_2 as i64 + d_2).rem_euclid(CONFIG_SIZE as i64) as usize;

                if !config.obstructed[new_alpha_1][new_alpha_2]
                    && troves[new_alpha_1][new_alpha_2]
                        .map_or(true, |t| t.distance > node.trove.distance + 1)
                {
                    queue.push_back(IndexedBFSTrove {
                        trove: BFSTrove {
                            previous: Some((node.alpha_1, node.alpha_2)),
                            distance: node.trove.distance + 1,
                        },
                        alpha_1: new_alpha_1,
                        alpha_2: new_alpha_2,
                    })
                }
            }
        }

        Self(troves)
    }

    pub fn path_to(&self, target: &na::Point2<f64>) -> Option<Vec<na::Point2<f64>>> {
        let mut current = self.0[target.x.to_degrees().rem_euclid(360.0).floor() as usize]
            [target.y.to_degrees().rem_euclid(360.0).floor() as usize]?;
        let mut path = vec![*target];

        while let Some(prev) = current.previous {
            current = self.0[prev.0][prev.1].unwrap();
            path.push(na::point![
                (prev.0 as f64 + 0.5).to_radians(),
                (prev.1 as f64 + 0.5).to_radians()
            ]);
        }

        path.reverse();
        Some(path)
    }
}

pub struct ConfigObstuction {
    obstructed: [[bool; CONFIG_SIZE]; CONFIG_SIZE],
    system: flat_chain::System,
    origin: na::Point2<f64>,
}

impl ConfigObstuction {
    pub fn new(system: flat_chain::System, origin: na::Point2<f64>) -> Self {
        let obstructed = [[false; CONFIG_SIZE]; CONFIG_SIZE];
        Self {
            system,
            origin,
            obstructed,
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

    pub fn texture(&self, access_map: &BFSMap) -> Texture {
        let mut texture = Texture::new_rgb(CONFIG_SIZE as u32, CONFIG_SIZE as u32);

        for (alpha_1, subarray) in self.obstructed.iter().enumerate() {
            for (alpha_2, &obstructed) in subarray.iter().enumerate() {
                texture.put(
                    alpha_1 as u32,
                    alpha_2 as u32,
                    Rgba([
                        0,
                        if obstructed { 255 } else { 0 },
                        255 - access_map.0[alpha_1 as usize][alpha_2 as usize]
                            .map_or(255, |t| t.distance.min(255) as u8),
                        255,
                    ]),
                );
            }
        }

        texture
    }
}
