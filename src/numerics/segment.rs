use super::Rect;
use nalgebra as na;

#[derive(PartialEq, Eq)]
pub enum Orientation {
    Collinear,
    Clockwise,
    Counterclockwise,
}

pub struct Segment(na::Point2<f64>, na::Point2<f64>);

impl Segment {
    pub fn new(p_1: na::Point2<f64>, p_2: na::Point2<f64>) -> Self {
        Self(p_1, p_2)
    }

    pub fn contains_point_collinear(&self, p: &na::Point2<f64>) -> bool {
        p.x <= f64::max(self.0.x, self.1.x)
            && p.x >= f64::min(self.0.x, self.1.x)
            && p.y <= f64::max(self.0.y, self.1.y)
            && p.y >= f64::min(self.0.y, self.1.y)
    }

    pub fn orientation(
        p_1: na::Point2<f64>,
        p_2: na::Point2<f64>,
        p_3: na::Point2<f64>,
    ) -> Orientation {
        match (p_2.y - p_1.y) * (p_3.x - p_2.x) - (p_2.x - p_1.x) * (p_3.y - p_2.y) {
            x if x > 0.0 => Orientation::Clockwise,
            x if x < 0.0 => Orientation::Counterclockwise,
            _ => Orientation::Collinear,
        }
    }

    pub fn intersects(&self, other: &Segment) -> bool {
        let o_1 = Self::orientation(self.0, self.1, other.0);
        let o_2 = Self::orientation(self.0, self.1, other.1);
        let o_3 = Self::orientation(other.0, other.1, self.0);
        let o_4 = Self::orientation(other.0, other.1, self.1);

        if o_1 != o_2 && o_3 != o_4 {
            return true;
        }

        if o_1 == Orientation::Collinear && self.contains_point_collinear(&other.0) {
            return true;
        }

        if o_2 == Orientation::Collinear && self.contains_point_collinear(&other.1) {
            return true;
        }

        if o_3 == Orientation::Collinear && other.contains_point_collinear(&self.0) {
            return true;
        }

        if o_4 == Orientation::Collinear && other.contains_point_collinear(&self.1) {
            return true;
        }

        false
    }

    pub fn collides_with_rect(&self, rect: &Rect) -> bool {
        let rect_side_1 = Segment::new(rect.p_1, na::point![rect.p_1.x, rect.p_2.y]);
        let rect_side_2 = Segment::new(rect.p_1, na::point![rect.p_2.x, rect.p_1.y]);
        let rect_side_3 = Segment::new(rect.p_2, na::point![rect.p_1.x, rect.p_2.y]);
        let rect_side_4 = Segment::new(rect.p_2, na::point![rect.p_2.x, rect.p_1.y]);

        let intersects_1 = self.intersects(&rect_side_1);
        let intersects_2 = self.intersects(&rect_side_2);
        let intersects_3 = self.intersects(&rect_side_3);
        let intersects_4 = self.intersects(&rect_side_4);
        let contains_1 = rect.contains_point(&self.0);
        let contains_2 = rect.contains_point(&self.1);

        intersects_1 || intersects_2 || intersects_3 || intersects_4 || contains_1 || contains_2
    }
}
