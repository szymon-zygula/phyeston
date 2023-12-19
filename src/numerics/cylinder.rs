use super::parametric::ParametricForm;
use nalgebra as na;

#[derive(Copy, Clone, Debug)]
pub struct Cylinder {
    pub radius: f64,
    pub length: f64,
}

impl Cylinder {
    pub fn new(radius: f64, length: f64) -> Self {
        Self { radius, length }
    }

    fn r_at(&self, vec: &na::Vector2<f64>) -> f64 {
        self.radius
            * if vec.y < 0.0 {
                10.0 * (vec.y + 0.1)
            } else if vec.y > 1.0 {
                10.0 * (1.1 - vec.y)
            } else {
                1.0
            }
    }
}

impl ParametricForm<2, 3> for Cylinder {
    fn bounds(&self) -> na::Vector2<(f64, f64)> {
        na::Vector2::new(
            (0.0, 2.0 * std::f64::consts::PI),
            (-0.1, 1.1), // [0.0, 1.0] for the walls, the rest for the tops
        )
    }

    fn wrapped(&self, dim: usize) -> bool {
        dim == 0
    }

    fn value(&self, vec: &na::Vector2<f64>) -> na::Point3<f64> {
        let r = self.r_at(vec);

        na::Point3::new(
            r * vec.x.cos(),
            r * vec.x.sin(),
            self.length * vec.y.clamp(0.0, 1.0),
        )
    }

    fn normal(&self, vec: &na::Vector2<f64>) -> na::Vector3<f64> {
        (if vec.y < 0.0 {
            na::vector![0.0, 0.0, -1.0]
        } else if vec.y > 1.0 {
            na::vector![0.0, 0.0, 1.0]
        } else {
            na::vector!(vec.x.cos(), vec.x.sin(), 0.0)
        })
        .normalize()
    }
}
