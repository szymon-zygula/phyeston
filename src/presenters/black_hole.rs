use super::{Presenter, PresenterBuilder};
use crate::{
    controls::mouse::MouseState,
    render::{
        gl_drawable::GlDrawable,
        gl_mesh::GlTriangleMesh,
        gl_program::GlProgram,
        mesh::{Mesh, Triangle},
    },
};
use egui::Ui;
use nalgebra as na;
use std::sync::Arc;

pub struct BlackHole {
    gl_program: GlProgram,
    rect_mesh: GlTriangleMesh,

    mass: f32,
}

impl BlackHole {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            gl_program: GlProgram::vertex_fragment(Arc::clone(&gl), "2d_vert", "pass_frag"),
            rect_mesh: Self::create_rect_mesh(Arc::clone(&gl)),

            mass: 1.0e9,
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

impl Presenter for BlackHole {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.label("Mass");
        ui.add(egui::widgets::Slider::new(&mut self.mass, 0.0..=1.0e15).logarithmic(true));
    }

    fn show_bottom_ui(&mut self, _ui: &mut Ui) {}

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        let Some(size) = size else { return };
        let aspect_ratio = size.width as f32 / size.height as f32;

        self.gl_program.enable();

        self.gl_program.uniform_matrix_4_f32_slice(
            "view_transform",
            na::matrix![
                1.0 / aspect_ratio, 0.0, 0.0, -0.3;
                0.0, 1.0, 0.0, 0.4;
                0.0, 0.0, 1.0, 0.0;
                0.0, 0.0, 0.0, 1.0;
            ]
            .as_slice(),
        );

        // Piston
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (na::geometry::Translation3::new(0.0, 0.0, 0.0).to_homogeneous()
                * na::geometry::Scale3::new(0.125, 0.125, 1.0).to_homogeneous())
            .as_slice(),
        );

        self.rect_mesh.draw();
    }

    fn update(&mut self, _delta: std::time::Duration) {}

    fn name(&self) -> &'static str {
        "Black Hole"
    }

    fn update_mouse(&mut self, _state: MouseState) {}
}

pub struct BlackHoleBuilder {}

impl BlackHoleBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for BlackHoleBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("Black Hole")
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(BlackHole::new(gl))
    }
}

impl Default for BlackHoleBuilder {
    fn default() -> Self {
        Self {}
    }
}
