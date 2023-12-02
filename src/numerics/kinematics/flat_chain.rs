use nalgebra as na;

pub enum ReverseSolutions {
    InfinitelyMany,
    Two(na::Vector2<f64>, na::Vector2<f64>),
    One(na::Vector2<f64>),
    None,
}

enum MiddleSolutions {
    Two(na::Vector2<f64>, na::Vector2<f64>),
    One(na::Vector2<f64>),
    None,
}

pub struct System {
    l_1: f64,
    l_2: f64,
}

impl System {
    pub fn new(l_1: f64, l_2: f64) -> Self {
        Self { l_1, l_2 }
    }

    pub fn reverse_kinematics(&self, target: &na::Vector2<f64>) -> ReverseSolutions {
        let x = target.x;
        let y = target.y;
        let x2 = x.powi(2);
        let y2 = y.powi(2);

        let p = -0.5 * (self.l_2.powi(2) - x2 - y2 - self.l_1.powi(2));

        if target.x != 0.0 {
            let middle = self.middle_from_target(x, y, x2, y2, p);

            match middle {
                MiddleSolutions::Two(s_1, s_2) => ReverseSolutions::Two(
                    self.middle_to_state(target, &s_1),
                    self.middle_to_state(target, &s_2),
                ),
                MiddleSolutions::One(s_0) => {
                    ReverseSolutions::One(self.middle_to_state(target, &s_0))
                }

                MiddleSolutions::None => ReverseSolutions::None,
            }
        } else if target.y != 0.0 {
            let middle = self.middle_from_target(y, x, y2, x2, p);

            match middle {
                MiddleSolutions::Two(s_1, s_2) => ReverseSolutions::Two(
                    self.middle_to_state(target, &na::vector![s_1.y, s_1.x]),
                    self.middle_to_state(target, &na::vector![s_2.y, s_2.x]),
                ),
                MiddleSolutions::One(s_0) => {
                    ReverseSolutions::One(self.middle_to_state(target, &na::vector![s_0.y, s_0.x]))
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
        let b = -2.0 * a * y / x2;
        let c = a.powi(2) / x2 - self.l_1.powi(2);

        let delta = b.powi(2) - 4.0 * a * c;

        if delta < 0.0 {
            MiddleSolutions::None
        } else if delta > 0.0 {
            let middle_y_1 = (-b - delta.sqrt()) / (2.0 * a);
            let middle_y_2 = (-b + delta.sqrt()) / (2.0 * a);
            let middle_x_1 = (p - y * middle_y_1) / x;
            let middle_x_2 = (p - y * middle_y_2) / x;

            MiddleSolutions::Two(
                na::vector![middle_x_1, middle_y_1],
                na::vector![middle_x_2, middle_x_2],
            )
        } else {
            let middle_y = -b / (2.0 * a);
            let middle_x = (p - y * middle_y) / x;

            MiddleSolutions::One(na::vector![middle_x, middle_y])
        }
    }

    fn middle_to_state(
        &self,
        target: &na::Vector2<f64>,
        middle: &na::Vector2<f64>,
    ) -> na::Vector2<f64> {
        let alpha_1 = f64::atan2(middle.y, middle.x);
        let target_in_frame_1 =
            na::Rotation2::new(-alpha_1).transform_vector(target) - na::vector![0.0, self.l_1];
        let alpha_2 = f64::atan2(target_in_frame_1.y, target_in_frame_1.x);

        na::vector![alpha_1, alpha_2]
    }
}
