use super::{Presenter, PresenterBuilder};
use crate::{
    controls::camera::Camera,
    numerics::{ode, RungeKuttaIV},
    render::{
        gl_drawable::GlDrawable,
        gl_program::GlProgram,
        mesh::{GlMesh, Mesh, Triangle},
        models,
    },
    simulators::spinning_top::SpinningTopODE,
};
use egui::{containers::ComboBox, Rgba, Slider, Ui};
use nalgebra as na;
use std::{f64::consts::PI, sync::Arc};

pub struct SpinningTop {
    meshes_program: GlProgram,
    box_mesh: GlMesh,
    plane_mesh: GlMesh,

    lines_program: GlProgram,
    gravity_mesh: GlMesh,
    path_mesh: GlMesh,

    camera: Camera,

    state: ode::State<7>,
    solver: RungeKuttaIV<7, SpinningTopODE>,

    density: f64,

    enable_gravity: bool,
    show_trajectory: bool,
    show_plane: bool,
    show_gravity_vector: bool,
    show_box: bool,
    show_diagonal: bool,

    max_trajectory_points: usize,
}

impl SpinningTop {
    const LIGHT_POSITION: na::Vector3<f32> = na::vector![3.0, 1.0, 1.0];
    const LIGHT_COLOR: na::Vector3<f32> = na::vector![1.0, 1.0, 1.0];
    const LIGHT_AMBIENT: na::Vector3<f32> = na::vector![0.2, 0.2, 0.2];

    const DEFAULT_DENSITY: f64 = 1.0;

    pub fn new(gl: Arc<glow::Context>) -> Self {
        let mut state = ode::State::<7> {
            t: 0.0,
            y: na::SVector::<f64, 7>::zeros(),
        };

        let rotation = na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);

        state.y[3] = rotation.w;
        state.y[4] = rotation.i;
        state.y[5] = rotation.j;
        state.y[6] = rotation.w;

        Self {
            meshes_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "perspective_vert",
                "phong_frag",
            ),
            box_mesh: GlMesh::new(Arc::clone(&gl), &models::cube()),
            plane_mesh: GlMesh::new(Arc::clone(&gl), &models::double_plane()),

            lines_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "perspective_vert",
                "color_frag",
            ),
            gravity_mesh: GlMesh::new(Arc::clone(&gl), &Mesh::<na::Point3<f32>>::empty()),
            path_mesh: GlMesh::new(Arc::clone(&gl), &Mesh::<na::Point3<f32>>::empty()),

            camera: Camera::new(),

            state,
            density: Self::DEFAULT_DENSITY,
            solver: RungeKuttaIV::new(0.01, SpinningTopODE::new(Self::DEFAULT_DENSITY)),

            enable_gravity: true,
            show_box: true,
            show_plane: true,
            show_gravity_vector: false,
            show_trajectory: false,
            show_diagonal: false,

            max_trajectory_points: 1000,
        }
    }

    fn box_transform(&self) -> na::Matrix4<f32> {
        let rotation = na::UnitQuaternion::new_normalize(na::Quaternion::new(
            self.state.y[3] as f32,
            self.state.y[4] as f32,
            self.state.y[5] as f32,
            self.state.y[6] as f32,
        ));

        let translation = na::Translation3::new(1.0, 1.0, 1.0);
        rotation.to_homogeneous() * translation.to_homogeneous()
    }

    pub fn draw_meshes(&self, aspect_ratio: f32) {
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

        if self.show_plane {
            self.draw_plane();
        }

        if self.show_box {
            self.draw_box();
        }
    }

    pub fn draw_box(&self) {
        self.meshes_program
            .uniform_4_f32("material_color", 0.2, 0.4, 0.8, 1.0);
        self.meshes_program.uniform_f32("material_diffuse", 0.8);
        self.meshes_program.uniform_f32("material_specular", 0.4);
        self.meshes_program
            .uniform_f32("material_specular_exp", 10.0);

        self.meshes_program
            .uniform_matrix_4_f32_slice("model_transform", self.box_transform().as_slice());

        self.box_mesh.draw();
    }

    pub fn draw_plane(&self) {
        self.meshes_program
            .uniform_4_f32("material_color", 0.8, 0.4, 0.2, 1.0);
        self.meshes_program.uniform_f32("material_diffuse", 0.8);
        self.meshes_program.uniform_f32("material_specular", 0.4);
        self.meshes_program
            .uniform_f32("material_specular_exp", 10.0);

        self.meshes_program
            .uniform_matrix_4_f32_slice("model_transform", na::Matrix4::identity().as_slice());

        self.plane_mesh.draw();
    }
}

impl Presenter for SpinningTop {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.enable_gravity, "Gravity");

        ui.checkbox(&mut self.show_plane, "Show plane");
        ui.checkbox(&mut self.show_gravity_vector, "Show gravity vector");
        ui.checkbox(&mut self.show_box, "Show box");
        ui.checkbox(&mut self.show_trajectory, "Show trajectory");
        ui.checkbox(&mut self.show_diagonal, "Show diagonal");

        ui.label("Maximum trajectory points visible");
        ui.add(egui::widgets::DragValue::new(
            &mut self.max_trajectory_points,
        ));

        ui.label("Box density");
        ui.add(egui::widgets::DragValue::new(&mut self.density));
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, aspect_ratio: f32) {
        self.draw_meshes(aspect_ratio);
    }

    fn update(&mut self) {}

    fn name(&self) -> &'static str {
        "Spinning Top"
    }
}

pub struct SpinningTopBuilder {}

impl SpinningTopBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for SpinningTopBuilder {
    fn build_ui(&mut self, ui: &mut Ui) {}

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(SpinningTop::new(gl))
    }
}

impl Default for SpinningTopBuilder {
    fn default() -> Self {
        Self {}
    }
}
