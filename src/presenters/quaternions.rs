use super::{Presenter, PresenterBuilder};
use crate::{
    controls::{camera::Camera, mouse::MouseState},
    numerics::rotations::*,
    render::{gl_drawable::GlDrawable, gl_program::GlProgram, mesh::GlTriangleMesh, models},
    ui::widgets,
};
use egui::{widgets::DragValue, Ui};
use glow::HasContext;
use na::SimdPartialOrd;
use nalgebra as na;
use std::sync::Arc;

pub struct Quaternions {
    camera: Camera,

    meshes_program: GlProgram,
    cube_mesh: GlTriangleMesh,
    gl: Arc<glow::Context>,

    start_rotation: Rotation,
    start_position: na::Vector3<f64>,
    end_rotation: Rotation,
    end_position: na::Vector3<f64>,
}

impl Quaternions {
    const LIGHT_POSITION: na::Vector3<f32> = na::vector![2.0, 4.0, 2.0];
    const LIGHT_COLOR: na::Vector3<f32> = na::vector![2.0, 2.0, 2.0];
    const LIGHT_AMBIENT: na::Vector3<f32> = na::vector![0.4, 0.4, 0.4];

    fn new(
        gl: Arc<glow::Context>,
        start_rotation: Rotation,
        start_position: na::Vector3<f64>,
        end_rotation: Rotation,
        end_position: na::Vector3<f64>,
    ) -> Self {
        Self {
            camera: Camera::new(),

            meshes_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "perspective_vert",
                "phong_frag",
            ),
            cube_mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::cube()),
            gl,

            start_rotation,
            start_position,
            end_rotation,
            end_position,
        }
    }

    fn draw_axis(&self, vector: &na::Vector3<f32>, color: &[f32; 4]) {
        let ones = na::vector![1.0, 1.0, 1.0];
        let scale = 0.6 * (ones * 0.1 + vector).simd_clamp(na::Vector3::zeros(), ones);
        let translation = 0.4 * vector;

        let transform = na::Translation3::from(translation).to_homogeneous()
            * na::Scale3::from(scale).to_homogeneous();

        self.meshes_program
            .uniform_4_f32_slice("material_color", color);
        self.meshes_program
            .uniform_matrix_4_f32_slice("model_transform", transform.as_slice());

        self.cube_mesh.draw();
    }

    fn draw_axes(&self) {
        self.meshes_program.uniform_f32("material_diffuse", 0.8);
        self.meshes_program.uniform_f32("material_specular", 0.4);
        self.meshes_program
            .uniform_f32("material_specular_exp", 10.0);

        self.draw_axis(&na::vector![1.0, 0.0, 0.0], &[1.0, 0.0, 0.0, 1.0]);
        self.draw_axis(&na::vector![0.0, 1.0, 0.0], &[0.0, 1.0, 0.0, 1.0]);
        self.draw_axis(&na::vector![0.0, 0.0, 1.0], &[0.0, 0.0, 1.0, 1.0]);
    }

    fn draw_meshes(&self, aspect_ratio: f32) {
        self.meshes_program.enable();
        self.meshes_program
            .uniform_matrix_4_f32_slice("view_transform", self.camera.view_transform().as_slice());
        self.meshes_program.uniform_matrix_4_f32_slice(
            "projection_transform",
            self.camera.projection_transform(aspect_ratio).as_slice(),
        );

        self.meshes_program
            .uniform_3_f32_slice("eye_position", self.camera.position().coords.as_slice());
        self.meshes_program
            .uniform_3_f32_slice("light_position", Self::LIGHT_POSITION.as_slice());
        self.meshes_program
            .uniform_3_f32_slice("light_color", Self::LIGHT_COLOR.as_slice());
        self.meshes_program
            .uniform_3_f32_slice("ambient", Self::LIGHT_AMBIENT.as_slice());

        self.draw_axes();
    }
}

impl Presenter for Quaternions {
    fn show_side_ui(&mut self, ui: &mut Ui) {}

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, aspect_ratio: f32) {
        self.draw_meshes(aspect_ratio)
    }

    fn update(&mut self, delta: std::time::Duration) {}

    fn update_mouse(&mut self, state: MouseState) {
        self.camera.update_from_mouse(state);
    }

    fn name(&self) -> &'static str {
        "Quaternions"
    }
}

#[derive(Default)]
pub struct QuaternionsBuilder {
    start_rotation: Rotation,
    start_position: na::Vector3<f64>,
    end_rotation: Rotation,
    end_position: na::Vector3<f64>,
}

impl QuaternionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn frame_ui(
        ui: &mut Ui,
        rotation: &mut Rotation,
        position: &mut na::Vector3<f64>,
    ) -> egui::Response {
        ui.label("Position")
            | widgets::vector_drag(ui, position, -10.0, 10.0, "", 0.1, &["x", "y", "z"])
            | ui.label("Rotation")
            | match rotation {
                Rotation::Quaternion(quaternion) => {
                    // Trick the borrow checker
                    let mut vector = &mut quaternion.0;
                    let mut dummy_vector = *vector;
                    if ui.button("Quaternion").clicked() {
                        vector = &mut dummy_vector;
                        *rotation = Rotation::EulerAngles(EulerAngles(na::Vector3::zeros()));
                    }

                    widgets::vector_drag(ui, vector, -1.0, 1.0, "", 0.01, &["x", "y", "z", "w"])
                }
                Rotation::EulerAngles(angles) => {
                    let mut vector = &mut angles.0;
                    let mut dummy_vector = *vector;

                    if ui.button("Euler angles").clicked() {
                        *rotation =
                            Rotation::Quaternion(Quaternion(na::Vector4::new(1.0, 0.0, 0.0, 0.0)));
                        vector = &mut dummy_vector;
                    }

                    widgets::vector_drag(ui, vector, 0.0, 360.0, "°", 1.0, &["x", "y", "z"])
                }
            }
    }
}

impl PresenterBuilder for QuaternionsBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("Start frame")
            | Self::frame_ui(ui, &mut self.start_rotation, &mut self.start_position)
            | ui.separator()
            | ui.label("End frame")
            | Self::frame_ui(ui, &mut self.end_rotation, &mut self.end_position)
            | ui.separator()
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(Quaternions::new(
            gl,
            self.start_rotation,
            self.start_position,
            self.end_rotation,
            self.end_position,
        ))
    }
}
