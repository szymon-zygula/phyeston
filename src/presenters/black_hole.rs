use super::{Presenter, PresenterBuilder};
use crate::{
    controls::{camera::Camera, mouse::MouseState},
    render::{
        gl_drawable::GlDrawable,
        gl_mesh::GlTriangleMesh,
        gl_program::GlProgram,
        gl_texture::GlCubeTexture,
        mesh::{Mesh, Triangle},
        models,
        texture::Texture,
    },
};
use egui::Ui;
use glow::HasContext;
use nalgebra as na;
use std::path::Path;
use std::sync::Arc;

pub struct BlackHole {
    gl: Arc<glow::Context>,
    gl_program: GlProgram,
    cube_texture: GlCubeTexture,
    skybox_cube: GlTriangleMesh,
    camera: Camera,

    mass: f32,
    fov: f32,
}

impl BlackHole {
    const ROOM_SCALE: f32 = 20.0;

    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            gl_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "black_hole_vert",
                "black_hole_frag",
            ),
            cube_texture: GlCubeTexture::new(
                Arc::clone(&gl),
                &[
                    Texture::from_file(&Path::new("textures/px.png")),
                    Texture::from_file(&Path::new("textures/nx.png")),
                    Texture::from_file(&Path::new("textures/py.png")),
                    Texture::from_file(&Path::new("textures/ny.png")),
                    Texture::from_file(&Path::new("textures/pz.png")),
                    Texture::from_file(&Path::new("textures/nz.png")),
                ],
            ),
            skybox_cube: GlTriangleMesh::new(Arc::clone(&gl), &models::cube()),
            camera: Camera::new(),

            mass: 1.0e9,
            fov: 70.0,
            gl,
        }
    }
}

impl Presenter for BlackHole {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.label("FOV");
        ui.add(egui::widgets::Slider::new(&mut self.fov, 0.0..=120.0));

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
                1.0 / aspect_ratio, 0.0, 0.0, 0.0;
                0.0, 1.0, 0.0, 0.0;
                0.0, 0.0, 1.0, 0.0;
                0.0, 0.0, 0.0, 1.0;
            ]
            .as_slice(),
        );

        self.gl_program.uniform_matrix_4_f32_slice(
            "view_transform",
            self.camera.view_transform_no_translation().as_slice(),
        );
        self.gl_program.uniform_matrix_4_f32_slice(
            "projection_transform",
            self.camera.projection_transform(aspect_ratio).as_slice(),
        );
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            na::geometry::Scale3::new(Self::ROOM_SCALE, Self::ROOM_SCALE, Self::ROOM_SCALE)
                .to_homogeneous()
                .as_slice(),
        );

        self.gl_program
            .uniform_3_f32_slice("eye_position", self.camera.position().coords.as_slice());

        self.cube_texture.bind();

        unsafe { self.gl.cull_face(glow::FRONT) };
        self.skybox_cube.draw();
        unsafe { self.gl.cull_face(glow::BACK) };
    }

    fn update(&mut self, _delta: std::time::Duration) {}

    fn name(&self) -> &'static str {
        "Black Hole"
    }

    fn update_mouse(&mut self, state: MouseState) {
        self.camera.update_from_mouse(state);
    }
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
