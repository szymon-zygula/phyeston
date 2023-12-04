use nalgebra as na;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub p_1: na::Point2<f64>,
    pub p_2: na::Point2<f64>,
}

impl Rect {
    pub fn contains_point(&self, p: &na::Point2<f64>) -> bool {
        ((self.p_1.x <= p.x && p.x <= self.p_2.x) || (self.p_2.x <= p.x && p.x <= self.p_1.x))
            && ((self.p_1.y <= p.y && p.y <= self.p_2.y)
                || (self.p_2.y <= p.y && p.y <= self.p_1.y))
    }
}
