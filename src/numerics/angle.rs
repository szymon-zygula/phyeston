#[derive(Copy, Clone, Debug)]
pub struct Angle(f64);

impl Angle {
    pub fn new(value: f64) -> Self {
        Self(value.rem_euclid(360.0))
    }

    pub fn rad(&self) -> f64 {
        self.0
    }

    pub fn deg(&self) -> f64 {
        self.0.to_degrees()
    }

    pub fn lerp(&self, mut other: Self, t: f64) -> Self {
        let mut me = *self;
        if (other.0 - me.0).abs() > 180.0 {
            if other.0 > me.0 {
                other.0 -= 360.0;
            } else {
                me.0 -= 360.0;
            }
        }

        Self(me.0 * (1.0 - t) + other.0 * t)
    }
}

impl std::ops::Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new((self.0 + rhs.0).rem_euclid(360.0))
    }
}

impl std::ops::Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new((self.0 - rhs.0).rem_euclid(360.0))
    }
}
