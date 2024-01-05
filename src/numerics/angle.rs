#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Angle(f64);

impl Angle {
    pub fn from_rad(value: f64) -> Self {
        Self(value.rem_euclid(std::f64::consts::PI * 2.0))
    }

    pub fn from_deg(value: f64) -> Self {
        Self(value.to_radians().rem_euclid(std::f64::consts::PI * 2.0))
    }

    pub fn rad(&self) -> f64 {
        self.0
    }

    pub fn deg(&self) -> f64 {
        self.0.to_degrees()
    }

    pub fn sin(&self) -> f64 {
        self.rad().sin()
    }

    pub fn cos(&self) -> f64 {
        self.rad().cos()
    }

    pub fn set_rad(&mut self, val: f64) {
        self.0 = val;
    }

    pub fn set_deg(&mut self, val: f64) {
        self.0 = val.to_radians();
    }

    pub fn lerp(&self, mut other: Self, t: f64) -> Self {
        let mut me = *self;
        if (other.0 - me.0).abs() > std::f64::consts::PI {
            if other.0 > me.0 {
                other.0 -= 2.0 * std::f64::consts::PI;
            } else {
                me.0 -= 2.0 * std::f64::consts::PI;
            }
        }

        Self(me.0 * (1.0 - t) + other.0 * t)
    }

    /// Distance between `self` and `other`
    pub fn dist(&self, other: Self) -> Self {
        Angle::from_rad(f64::min((*self - other).0, (other - *self).0))
    }

    /// Returns either `a_0` or `a_1`, depending on which one is closer to `self`
    pub fn closest(&self, a_0: Self, a_1: Self) -> Self {
        let diff_0 = self.dist(a_0);
        let diff_1 = self.dist(a_1);

        if diff_0 <= diff_1 {
            a_0
        } else {
            a_1
        }
    }

    pub fn pi_rad() -> Self {
        Self::from_rad(std::f64::consts::PI)
    }
}

impl std::ops::Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_rad(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_rad(self.0 - rhs.0)
    }
}

impl std::ops::Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_rad(-self.0)
    }
}
