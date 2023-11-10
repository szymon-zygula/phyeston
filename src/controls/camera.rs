use super::mouse::MouseState;
use egui_winit::winit::dpi::{PhysicalPosition, PhysicalSize};
use nalgebra as na;

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub azimuth: f32,
    pub altitude: f32,
    pub log_distance: f32,
    pub center: na::Point3<f32>,
    pub resolution: PhysicalSize<u32>,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Camera {
    const ROTATION_SPEED: f32 = 0.05;
    const MOVEMENT_SPEED: f32 = 0.01;
    const SCROLL_SPEED: f32 = 0.2;

    pub fn new() -> Camera {
        Camera {
            azimuth: -std::f32::consts::FRAC_PI_4,
            altitude: std::f32::consts::FRAC_PI_4,
            log_distance: 2.0,
            center: na::Point3::new(0.0, 0.0, 0.0),
            resolution: PhysicalSize::new(0, 0),
            near_plane: 0.1,
            far_plane: 10000.0,
        }
    }

    pub fn linear_distance(&self) -> f32 {
        self.log_distance.exp()
    }

    pub fn set_linear_distance(&mut self, linear_distance: f32) {
        self.log_distance = linear_distance.ln();
    }

    pub fn update_from_mouse(&mut self, mouse: &mut MouseState) -> bool {
        let mouse_delta = mouse.position_delta();
        let scroll_delta = mouse.scroll_delta();

        if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 || scroll_delta != 0.0 {
            self.update_angles(mouse, &mouse_delta);
            self.update_center(mouse, &mouse_delta);

            self.log_distance -= Self::SCROLL_SPEED * scroll_delta;
            self.log_distance = self
                .log_distance
                .clamp(self.near_plane.ln(), self.far_plane.ln());

            true
        } else {
            false
        }
    }

    fn update_angles(&mut self, mouse: &MouseState, mouse_delta: &PhysicalPosition<f64>) {
        if mouse.is_middle_button_down() {
            self.azimuth += mouse_delta.x as f32 * Self::ROTATION_SPEED;
            self.altitude += mouse_delta.y as f32 * Self::ROTATION_SPEED;
        }
    }

    fn update_center(&mut self, mouse: &MouseState, mouse_delta: &PhysicalPosition<f64>) {
        if mouse.is_right_button_down() {
            self.center += (na::geometry::Rotation3::from_axis_angle(
                &na::Unit::new_normalize(na::vector![0.0, 1.0, 0.0]),
                -self.azimuth,
            )
            .to_homogeneous()
                * na::geometry::Rotation3::from_axis_angle(
                    &na::Unit::new_normalize(na::vector![1.0, 0.0, 0.0]),
                    -self.altitude,
                )
                .to_homogeneous()
                * na::Vector4::new(-mouse_delta.x as f32, mouse_delta.y as f32, 0.0, 0.0))
            .xyz()
                * self.linear_distance()
                * Self::MOVEMENT_SPEED;
        }
    }

    pub fn position(&self) -> na::Point3<f32> {
        let homogeneous_position =
            self.inverse_view_transform() * na::Point4::new(0.0, 0.0, 0.0, 1.0);
        na::Point3::from_homogeneous(homogeneous_position.coords).unwrap()
    }

    pub fn view_transform(&self) -> na::Matrix4<f32> {
        na::Translation3::new(0.0, 0.0, -self.linear_distance()).to_homogeneous()
            * na::Rotation3::from_axis_angle(
                &na::Unit::new_normalize(na::vector![1.0, 0.0, 0.0]),
                self.altitude,
            )
            .to_homogeneous()
            * na::Rotation3::from_axis_angle(
                &na::Unit::new_normalize(na::vector![0.0, 1.0, 0.0]),
                self.azimuth,
            )
            .to_homogeneous()
            * na::Translation3::from(-self.center.coords).to_homogeneous()
    }

    pub fn inverse_view_transform(&self) -> na::Matrix4<f32> {
        na::Translation3::from(self.center.coords).to_homogeneous()
            * na::Rotation3::from_axis_angle(
                &na::Unit::new_normalize(na::vector![0.0, 1.0, 0.0]),
                -self.azimuth,
            )
            .to_homogeneous()
            * na::Rotation3::from_axis_angle(
                &na::Unit::new_normalize(na::vector![1.0, 0.0, 0.0]),
                -self.altitude,
            )
            .to_homogeneous()
            * na::Translation3::new(0.0, 0.0, self.linear_distance()).to_homogeneous()
    }

    pub fn projection_transform(&self, aspect: f32) -> na::Matrix4<f32> {
        na::Perspective3::new(
            aspect,
            std::f32::consts::FRAC_2_PI,
            self.near_plane,
            self.far_plane,
        )
        .to_homogeneous()
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.resolution.width as f32 / self.resolution.height as f32
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
