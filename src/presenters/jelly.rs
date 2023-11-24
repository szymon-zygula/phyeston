use super::Presenter;
use super::PresenterBuilder;
use crate::controls::{camera::Camera, mouse::MouseState};
use crate::numerics::bezier;
use crate::render::{
    gl_drawable::GlDrawable,
    gl_mesh::{GlLineStrip, GlLines, GlPointCloud, GlTriangleMesh},
    gl_program::GlProgram,
    mesh::Mesh,
    models,
};
use crate::ui::widgets::vector_drag;
use egui::Ui;
use glow::HasContext;
use itertools::Itertools;
use nalgebra as na;
use std::path::Path;
use std::sync::Arc;

const ROOM_COLOR: na::Vector4<f32> = na::vector![0.8, 0.4, 0.2, 0.4];

const LIGHT_POSITION: na::Vector3<f32> = na::vector![-2.0, 4.0, -2.0];
const LIGHT_COLOR: na::Vector3<f32> = na::vector![1.0, 1.0, 1.0];
const LIGHT_AMBIENT: na::Vector3<f32> = na::vector![0.4, 0.4, 0.4];

struct Room {
    program: GlProgram,
    mesh: GlTriangleMesh,
    transform: na::Matrix4<f32>,
    show: bool,
}

impl Room {
    fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            program: GlProgram::vertex_fragment(Arc::clone(&gl), "perspective_vert", "phong_frag"),
            mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::inverse_cube()),
            transform: na::Scale3::new(5.0, 5.0, 5.0).to_homogeneous(),
            show: true,
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.show, "Show the room");
    }

    fn draw(&self, aspect_ratio: f32, camera: &Camera) {
        if !self.show {
            return;
        }

        self.program.enable();
        self.program
            .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        self.program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform(aspect_ratio).as_slice(),
        );

        self.program
            .uniform_3_f32_slice("eye_position", camera.position().coords.as_slice());
        self.program
            .uniform_3_f32_slice("light_position", LIGHT_POSITION.as_slice());
        self.program
            .uniform_3_f32_slice("light_color", LIGHT_COLOR.as_slice());
        self.program
            .uniform_3_f32_slice("ambient", LIGHT_AMBIENT.as_slice());

        self.program
            .uniform_4_f32_slice("material_color", ROOM_COLOR.as_slice());
        self.program.uniform_f32("material_diffuse", 0.8);
        self.program.uniform_f32("material_specular", 0.4);
        self.program.uniform_f32("material_specular_exp", 10.0);

        self.program
            .uniform_matrix_4_f32_slice("model_transform", self.transform.as_slice());

        self.mesh.draw();
    }
}

struct ControlFrame {
    program: GlProgram,
    strip: GlLineStrip,
    translation: na::Vector3<f64>,
    rotation: na::Quaternion<f64>,
    transform: na::Matrix4<f32>,
    show: bool,
}

impl ControlFrame {
    fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            program: GlProgram::vertex_fragment(Arc::clone(&gl), "perspective_vert", "color_frag"),
            strip: GlLineStrip::new(Arc::clone(&gl), &models::wire_cube()),
            translation: na::Vector3::zeros(),
            rotation: na::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            transform: na::Matrix4::identity(),
            show: true,
        }
    }

    fn recalculate_transform(&mut self) {
        self.transform = na::Translation3::from(self.translation)
            .to_homogeneous()
            .map(|c| c as f32)
            * na::UnitQuaternion::new_normalize(self.rotation)
                .to_homogeneous()
                .map(|c| c as f32);
    }

    fn draw(&self, aspect_ratio: f32, camera: &Camera) {
        if !self.show {
            return;
        }

        self.program.enable();
        self.program
            .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        self.program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform(aspect_ratio).as_slice(),
        );
        self.program
            .uniform_matrix_4_f32_slice("model_transform", self.transform.as_slice());
        self.program.uniform_4_f32("color", 0.0, 0.0, 0.0, 1.0);

        self.strip.draw();
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.show, "Show control frame");
        ui.label("Control frame position");

        let result = vector_drag(
            ui,
            &mut self.translation,
            -10.0,
            10.0,
            "",
            0.05,
            &["x", "y", "z"],
        ) | vector_drag(
            ui,
            &mut self.rotation.coords,
            -1.0,
            1.0,
            "",
            0.01,
            &["x", "y", "z", "w"],
        );

        if result.changed() {
            self.recalculate_transform();
        }
    }
}

struct Model {
    program: GlProgram,
    mesh: GlTriangleMesh,
    transform: na::Matrix4<f32>,
    show: bool,
}

impl Model {
    const MODEL_COLOR: [f32; 4] = [0.1, 0.4, 1.0, 1.0];
    fn new(gl: Arc<glow::Context>) -> Self {
        let cube = bezier::Cube::new();
        Self {
            program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "bezier_deformed_vert",
                "phong_frag",
            ),
            mesh: GlTriangleMesh::new(
                Arc::clone(&gl),
                &Mesh::from_file(Path::new("models/duck.txt")),
            ),
            transform: na::Translation3::new(0.5, 0.0, 0.5).to_homogeneous()
                * na::Scale3::new(0.005, 0.005, 0.005).to_homogeneous(),
            show: true,
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.show, "Show model");
    }

    fn draw(&self, aspect_ratio: f32, camera: &Camera, cube: &[f32; 3 * 64]) {
        if !self.show {
            return;
        }

        self.program.enable();
        self.program
            .uniform_matrix_4_f32_slice("view", camera.view_transform().as_slice());
        self.program.uniform_matrix_4_f32_slice(
            "projection",
            camera.projection_transform(aspect_ratio).as_slice(),
        );
        self.program
            .uniform_matrix_4_f32_slice("model", self.transform.as_slice());
        self.program.uniform_3_f32_slice("bezier_cube", cube);

        self.program
            .uniform_3_f32_slice("eye_position", camera.position().coords.as_slice());
        self.program
            .uniform_3_f32_slice("light_position", LIGHT_POSITION.as_slice());
        self.program
            .uniform_3_f32_slice("light_color", LIGHT_COLOR.as_slice());
        self.program
            .uniform_3_f32_slice("ambient", LIGHT_AMBIENT.as_slice());

        self.program
            .uniform_4_f32_slice("material_color", Self::MODEL_COLOR.as_slice());
        self.program.uniform_f32("material_diffuse", 0.8);
        self.program.uniform_f32("material_specular", 0.4);
        self.program.uniform_f32("material_specular_exp", 10.0);

        self.mesh.draw();
    }
}

struct BezierCube {
    point_program: GlProgram,
    point_cloud: GlPointCloud,
    show_points: bool,

    grid_program: GlProgram,
    grid_lines: GlLines,
    grid_transform: na::Matrix4<f32>, // Cached identity
    show_grid: bool,

    cube: bezier::Cube<f64>,
    flat_cube: [f32; 3 * 64],
    gl: Arc<glow::Context>,
}

impl BezierCube {
    const POINT_SIZE: f32 = 6.0;
    const POINT_COLOR: [f32; 4] = [0.4, 1.0, 0.4, 1.0];

    fn new(gl: Arc<glow::Context>) -> Self {
        let cube = bezier::Cube::new();
        Self {
            point_program: GlProgram::vertex_fragment(Arc::clone(&gl), "point_vert", "color_frag"),
            point_cloud: GlPointCloud::new(Arc::clone(&gl), &cube.as_f32_array()),
            show_points: true,

            grid_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "perspective_vert",
                "color_frag",
            ),
            grid_lines: GlLines::new(Arc::clone(&gl), &models::wire_grid()),
            grid_transform: na::Matrix4::identity(),
            show_grid: true,

            flat_cube: cube.as_f32_flat(),
            cube,
            gl,
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.show_points, "Show bezier points");
        ui.checkbox(&mut self.show_grid, "Show bezier grid");
    }

    fn draw_points(&self, aspect_ratio: f32, camera: &Camera) {
        unsafe { self.gl.enable(glow::PROGRAM_POINT_SIZE) };

        self.point_program.enable();

        self.point_program
            .uniform_f32("point_size", Self::POINT_SIZE);
        self.point_program
            .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        self.point_program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform(aspect_ratio).as_slice(),
        );
        self.point_program
            .uniform_4_f32_slice("color", &Self::POINT_COLOR);

        self.point_cloud.draw();
    }

    fn draw_grid(&self, aspect_ratio: f32, camera: &Camera) {
        self.grid_program.enable();
        self.grid_program
            .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        self.grid_program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform(aspect_ratio).as_slice(),
        );
        self.grid_program
            .uniform_matrix_4_f32_slice("model_transform", self.grid_transform.as_slice());
        self.grid_program.uniform_4_f32("color", 0.0, 0.0, 0.0, 1.0);

        self.grid_lines.draw();
    }

    fn draw(&self, aspect_ratio: f32, camera: &Camera) {
        if self.show_points {
            self.draw_points(aspect_ratio, camera);
        }

        if self.show_grid {
            self.draw_grid(aspect_ratio, camera);
        }
    }

    fn update_cube(&mut self) {
        self.flat_cube = self.cube.as_f32_flat();
        let cube_array = self.cube.as_f32_array();
        self.point_cloud.update_points(&cube_array);
    }
}

pub struct Jelly {
    camera: Camera,

    bezier_cube: BezierCube,
    model: Model,
    room: Room,
    control_frame: ControlFrame,

    gl: Arc<glow::Context>,
}

impl Jelly {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            camera: Camera::new(),

            bezier_cube: BezierCube::new(Arc::clone(&gl)),
            model: Model::new(Arc::clone(&gl)),
            room: Room::new(Arc::clone(&gl)),
            control_frame: ControlFrame::new(Arc::clone(&gl)),

            gl,
        }
    }
}

impl Presenter for Jelly {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        self.bezier_cube.ui(ui);
        self.model.ui(ui);
        self.control_frame.ui(ui);
        self.room.ui(ui);
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        let Some(size) = size else { return };
        let aspect_ratio = size.width as f32 / size.height as f32;

        self.bezier_cube.draw(aspect_ratio, &self.camera);
        self.model
            .draw(aspect_ratio, &self.camera, &self.bezier_cube.flat_cube);
        self.control_frame.draw(aspect_ratio, &self.camera);
        self.room.draw(aspect_ratio, &self.camera);
    }

    fn update(&mut self, delta: std::time::Duration) {}

    fn update_mouse(&mut self, state: MouseState) {
        self.camera.update_from_mouse(state);
    }

    fn name(&self) -> &'static str {
        "Jelly"
    }
}

#[derive(Default)]
pub struct JellyBuilder {}

impl JellyBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for JellyBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("")
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(Jelly::new(gl))
    }
}
