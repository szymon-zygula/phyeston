use nalgebra as na;

#[derive(Debug)]
pub enum ReverseSolutions {
    InfinitelyMany,
    Two(na::Point2<f64>, na::Point2<f64>),
    One(na::Point2<f64>),
    None,
}

enum MiddleSolutions {
    Two(na::Point2<f64>, na::Point2<f64>),
    One(na::Point2<f64>),
    None,
}

pub struct State {
    pub p_1: na::Point2<f64>,
    pub p_2: na::Point2<f64>,
}

pub struct System {
    pub l_1: f64,
    pub l_2: f64,
}

impl System {
    pub fn new(l_1: f64, l_2: f64) -> Self {
        Self { l_1, l_2 }
    }

    pub fn forward_kinematics(&self, config_state: &na::Point2<f64>) -> State {
        let p_1 = na::Point2::origin()
            + self.l_1 * na::vector![config_state.x.cos(), config_state.x.sin()];
        let p_2 = p_1
            + self.l_2
                * na::vector![
                    (config_state.x + config_state.y).cos(),
                    (config_state.x + config_state.y).sin()
                ];
        State { p_1, p_2 }
    }

    pub fn inverse_kinematics(&self, target: &na::Point2<f64>) -> ReverseSolutions {
        let x = target.x;
        let y = target.y;
        let x2 = x.powi(2);
        let y2 = y.powi(2);

        let p = -0.5 * (self.l_2.powi(2) - x2 - y2 - self.l_1.powi(2));

        if target.x != 0.0 {
            let middle = self.middle_from_target(x, y, x2, y2, p);

            match middle {
                MiddleSolutions::Two(s_1, s_2) => ReverseSolutions::Two(
                    self.to_config_state(target, &s_1),
                    self.to_config_state(target, &s_2),
                ),
                MiddleSolutions::One(s_0) => {
                    ReverseSolutions::One(self.to_config_state(target, &s_0))
                }

                MiddleSolutions::None => ReverseSolutions::None,
            }
        } else if target.y != 0.0 {
            let middle = self.middle_from_target(y, x, y2, x2, p);

            match middle {
                MiddleSolutions::Two(s_1, s_2) => ReverseSolutions::Two(
                    self.to_config_state(target, &na::point![s_1.y, s_1.x]),
                    self.to_config_state(target, &na::point![s_2.y, s_2.x]),
                ),
                MiddleSolutions::One(s_0) => {
                    ReverseSolutions::One(self.to_config_state(target, &na::point![s_0.y, s_0.x]))
                }
                MiddleSolutions::None => ReverseSolutions::None,
            }
        } else {
            if self.l_1 == self.l_2 {
                ReverseSolutions::InfinitelyMany
            } else {
                ReverseSolutions::None
            }
        }
    }

    fn middle_from_target(&self, x: f64, y: f64, x2: f64, y2: f64, p: f64) -> MiddleSolutions {
        let a = 1.0 + y2 / x2;
        let b = -2.0 * p * y / x2;
        let c = p.powi(2) / x2 - self.l_1.powi(2);

        let delta = b.powi(2) - 4.0 * a * c;

        if delta < 0.0 {
            MiddleSolutions::None
        } else if delta > 0.0 {
            let middle_y_1 = (-b - delta.sqrt()) / (2.0 * a);
            let middle_y_2 = (-b + delta.sqrt()) / (2.0 * a);
            let middle_x_1 = (p - y * middle_y_1) / x;
            let middle_x_2 = (p - y * middle_y_2) / x;

            MiddleSolutions::Two(
                na::point![middle_x_1, middle_y_1],
                na::point![middle_x_2, middle_y_2],
            )
        } else {
            let middle_y = -b / (2.0 * a);
            let middle_x = (p - y * middle_y) / x;

            MiddleSolutions::One(na::point![middle_x, middle_y])
        }
    }

    fn to_config_state(
        &self,
        target: &na::Point2<f64>,
        middle: &na::Point2<f64>,
    ) -> na::Point2<f64> {
        let alpha_1 = f64::atan2(middle.y, middle.x);
        let target_in_frame_1 =
            na::Rotation2::new(-alpha_1).transform_point(&(target - middle).into());
        let alpha_2 = f64::atan2(target_in_frame_1.y, target_in_frame_1.x);

        na::point![alpha_1, alpha_2]
    }
}
