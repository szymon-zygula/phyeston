use super::{Presenter, PresenterBuilder};
use crate::{
    controls::mouse::MouseState,
    render::{
        gl_drawable::GlDrawable,
        gl_mesh::{GlLineStrip, GlLines, GlTriangleMesh},
        gl_program::GlProgram,
        mesh::{Mesh, Triangle},
        models,
    },
};
use egui::Ui;
use nalgebra as na;
use std::sync::Arc;

pub struct Hodograph {
    gl_program: GlProgram,
    rect_mesh: GlTriangleMesh,
    circle_mesh: GlLineStrip,
    arm_mesh: GlLines,
    radius_mesh: GlLines,

    stddev: f64,
    angular_speed: f64,
    arm_length: f64,
    wheel_radius: f64,

    angle: f64,

    simulation_speed: f64,
}

impl Hodograph {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Hodograph {
            rect_mesh: Self::create_rect_mesh(Arc::clone(&gl)),
            circle_mesh: GlLineStrip::new(Arc::clone(&gl), &models::circle(100)),
            radius_mesh: GlLines::new(
                Arc::clone(&gl),
                &[na::Point3::origin(), na::Point3::origin()],
            ),
            arm_mesh: GlLines::new(
                Arc::clone(&gl),
                &[na::Point3::origin(), na::Point3::origin()],
            ),
            gl_program: GlProgram::vertex_fragment(Arc::clone(&gl), "2d_vert", "pass_frag"),

            stddev: 1.0,
            angular_speed: 1.0,
            arm_length: 5.0,
            wheel_radius: 0.5,

            angle: 0.0,

            simulation_speed: 0.1,
        }
    }

    fn create_rect_mesh(gl: Arc<glow::Context>) -> GlTriangleMesh {
        // 0 1
        // 3 2
        let mesh = Mesh::new(
            vec![
                na::point!(-0.25, 0.25, 0.0),
                na::point!(0.25, 0.25, 0.0),
                na::point!(0.25, -0.25, 0.0),
                na::point!(-0.25, -0.25, 0.0),
            ],
            vec![Triangle([2, 1, 0]), Triangle([3, 2, 0])],
        );
        GlTriangleMesh::new(gl, &mesh)
    }
}

impl Presenter for Hodograph {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.label("ε0");
        ui.add(
            egui::DragValue::new(&mut self.stddev)
                .clamp_range(0.0..=10.0)
                .speed(0.1),
        );

        ui.label("ω");
        ui.add(
            egui::DragValue::new(&mut self.angular_speed)
                .clamp_range(-10.0..=10.0)
                .speed(0.1),
        );

        ui.label("L");
        ui.add(
            egui::DragValue::new(&mut self.arm_length)
                .clamp_range(0.0..=10.0)
                .speed(0.1),
        );

        ui.label("R");
        ui.add(
            egui::DragValue::new(&mut self.wheel_radius)
                .clamp_range(0.0..=1.0)
                .speed(0.01),
        );

        ui.label("Simulation Speed");
        ui.add(
            egui::DragValue::new(&mut self.simulation_speed)
                .clamp_range(0.0..=10.0)
                .speed(0.1),
        );
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {}

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        let Some(size) = size else { return };
        let aspect_ratio = size.width as f32 / size.height as f32;

        self.gl_program.enable();

        self.gl_program.uniform_matrix_4_f32_slice(
            "view_transform",
            na::matrix![
                1.0 / aspect_ratio, 0.0, 0.0, 0.0;
                0.0, 1.0, 0.0, 0.0;
                0.0, 0.0, 1.0, 0.0;
                0.0, 0.0, 0.0, 1.0;
            ]
            .as_slice(),
        );

        // Piston
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (na::geometry::Translation3::new(1.0, 0.0, 0.0).to_homogeneous()
                * na::geometry::Scale3::new(0.5, 0.5, 1.0).to_homogeneous())
            .as_slice(),
        );

        self.rect_mesh.draw();

        // Wheel
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (na::geometry::Translation3::new(-0.0, 0.0, 0.0).to_homogeneous()
                * na::geometry::Scale3::new(
                    self.wheel_radius as f32,
                    self.wheel_radius as f32,
                    1.0,
                )
                .to_homogeneous())
            .as_slice(),
        );

        self.circle_mesh.draw();

        // Lines
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", na::Matrix4::identity().as_slice());
        self.radius_mesh.draw();
        self.arm_mesh.draw();
    }

    fn update(&mut self, _delta: std::time::Duration) {}

    fn name(&self) -> &'static str {
        "Hodograph"
    }

    fn update_mouse(&mut self, _state: MouseState) {}
}

pub struct HodographBuilder {}

impl HodographBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for HodographBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("Hodograph")
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(Hodograph::new(gl))
    }
}

impl Default for HodographBuilder {
    fn default() -> Self {
        Self {}
    }
}
