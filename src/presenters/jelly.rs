use super::Presenter;
use super::PresenterBuilder;
use crate::controls::{camera::Camera, mouse::MouseState};
use crate::numerics::{bezier, ode};
use crate::render::{
    gl_drawable::GlDrawable,
    gl_mesh::{GlLineStrip, GlLines, GlPointCloud, GlTesselationBicubicPatch, GlTriangleMesh},
    gl_program::GlProgram,
    mesh::Mesh,
    models,
};
use crate::simulators::jelly::{self, JellyODE, JellyState};
use crate::ui::widgets::vector_drag;
use egui::{DragValue, Ui};
use glow::HasContext;
use nalgebra as na;
use rand::Rng;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

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
    const COLOR: na::Vector4<f32> = na::vector![0.8, 0.4, 0.2, 0.4];

    fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            program: GlProgram::vertex_fragment(Arc::clone(&gl), "perspective_vert", "phong_frag"),
            mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::inverse_cube()),
            transform: na::Scale3::new(
                jelly::ROOM_HALF_SIZE as f32,
                jelly::ROOM_HALF_SIZE as f32,
                jelly::ROOM_HALF_SIZE as f32,
            )
            .to_homogeneous(),
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
            .uniform_4_f32_slice("material_color", Self::COLOR.as_slice());
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
    transform: Rc<RefCell<jelly::ControlFrameTransform>>,
    composed_transform: na::Matrix4<f32>,
    show: bool,
}

impl ControlFrame {
    fn new(gl: Arc<glow::Context>, transform: Rc<RefCell<jelly::ControlFrameTransform>>) -> Self {
        Self {
            program: GlProgram::vertex_fragment(Arc::clone(&gl), "perspective_vert", "color_frag"),
            strip: GlLineStrip::new(Arc::clone(&gl), &models::wire_cube()),
            transform,
            composed_transform: na::Matrix4::identity(),
            show: true,
        }
    }

    fn recalculate_transform(&mut self) {
        let transform = self.transform.borrow();
        self.composed_transform = na::Translation3::from(transform.translation)
            .to_homogeneous()
            .map(|c| c as f32)
            * na::UnitQuaternion::new_normalize(transform.rotation)
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
            .uniform_matrix_4_f32_slice("model_transform", self.composed_transform.as_slice());
        self.program.uniform_4_f32("color", 0.0, 0.0, 0.0, 1.0);

        self.strip.draw();
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.show, "Show control frame");
        ui.label("Control frame position");

        let mut transform = self.transform.borrow_mut();

        let result = vector_drag(
            ui,
            &mut transform.translation,
            -10.0,
            10.0,
            "",
            0.05,
            &["x", "y", "z"],
        ) | vector_drag(
            ui,
            &mut transform.rotation.coords,
            -1.0,
            1.0,
            "",
            0.01,
            &["x", "y", "z", "w"],
        );

        drop(transform);

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
        self.grid_lines
            .update_points(&models::wire_grid_from_fn(|u, v, w| {
                self.cube.0[u][v][w].map(|c| (c + 1.0) as f32 * 0.5)
            }))
    }
}

struct BezierPatches {
    program: GlProgram,
    surfaces: [GlTesselationBicubicPatch; 6],
    show: bool,
    gl: Arc<glow::Context>,
}

impl BezierPatches {
    const SUBDIVISIONS: u32 = 16;
    const COLOR: [f32; 4] = [1.0, 0.2, 0.2, 1.0];

    fn new(gl: Arc<glow::Context>, cube: &bezier::Cube<f64>) -> Self {
        Self {
            program: GlProgram::with_shader_names(
                Arc::clone(&gl),
                &[
                    ("bezier_vert", glow::VERTEX_SHADER),
                    ("bezier_tsct", glow::TESS_CONTROL_SHADER),
                    ("bezier_tsev", glow::TESS_EVALUATION_SHADER),
                    ("phong_frag", glow::FRAGMENT_SHADER),
                ],
            ),
            surfaces: cube
                .patches_f32()
                .map(|p| GlTesselationBicubicPatch::new(Arc::clone(&gl), &p)),
            show: true,
            gl,
        }
    }

    fn draw(&self, aspect_ratio: f32, camera: &Camera) {
        if !self.show {
            return;
        }

        self.program.enable();
        self.program
            .uniform_u32("u_subdivisions", Self::SUBDIVISIONS);
        self.program
            .uniform_u32("v_subdivisions", Self::SUBDIVISIONS);

        self.program
            .uniform_matrix_4_f32_slice("view", camera.view_transform().as_slice());
        self.program.uniform_matrix_4_f32_slice(
            "projection",
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
            .uniform_4_f32_slice("material_color", Self::COLOR.as_slice());
        self.program.uniform_f32("material_diffuse", 0.8);
        self.program.uniform_f32("material_specular", 0.4);
        self.program.uniform_f32("material_specular_exp", 10.0);

        self.program.uniform_u32("invert_normals", 0);
        for surface in self.surfaces.iter().take(3) {
            surface.draw();
        }

        unsafe { self.gl.cull_face(glow::FRONT) };
        self.program.uniform_u32("invert_normals", 1);
        for surface in self.surfaces.iter().skip(3).take(3) {
            surface.draw();
        }
        unsafe { self.gl.cull_face(glow::BACK) };
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.show, "Show bezier patches");
    }

    fn update_cube(&mut self, cube: &bezier::Cube<f64>) {
        self.surfaces = cube
            .patches_f32()
            .map(|p| GlTesselationBicubicPatch::new(Arc::clone(&self.gl), &p));
    }
}

struct Simulation {
    state: JellyState,
    solver: Box<dyn ode::SolverWithDelta<{ jelly::ODE_DIM }, JellyODE>>,
    disruption_strength: f64,
    simulation_speed: f64,
    exact_t: f64,
}

impl Simulation {
    fn new(control_frame_transform: Rc<RefCell<jelly::ControlFrameTransform>>) -> Self {
        Self {
            state: JellyODE::default_state(),
            solver: Box::new(ode::RungeKuttaIV::new(
                0.01,
                JellyODE::new(control_frame_transform),
            )),
            disruption_strength: 1.0,
            simulation_speed: 1.0,
            exact_t: 0.0,
        }
    }

    fn update(
        &mut self,
        cube: &mut BezierCube,
        patches: &mut BezierPatches,
        delta: std::time::Duration,
    ) {
        let elapsed_t = delta.as_secs_f64() * self.simulation_speed;
        self.exact_t += elapsed_t;

        while self.exact_t > self.state.t {
            self.step_update(cube, patches);
        }
    }

    fn step_update(&mut self, cube: &mut BezierCube, patches: &mut BezierPatches) {
        self.state = self
            .solver
            .ode()
            .apply_collisions(self.solver.step(&self.state));

        for idx in 0..jelly::POINT_COUNT {
            let point = cube.cube.flat_mut(idx);
            point.x = self.state.y[idx * 3 + 0];
            point.y = self.state.y[idx * 3 + 1];
            point.z = self.state.y[idx * 3 + 2];
        }

        cube.update_cube();
        patches.update_cube(&cube.cube);
    }

    fn apply_random_disruption(&mut self) {
        let mut rng = rand::thread_rng();
        for y in self
            .state
            .y
            .iter_mut()
            .skip(jelly::SPACE_DIM)
            .take(jelly::ODE_DIM)
        {
            *y += (rng.gen::<f64>() * 2.0 - 1.0) * self.disruption_strength;
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.label("Simulation speed");
        ui.add(
            DragValue::new(&mut self.simulation_speed)
                .clamp_range(0.0..=f64::MAX)
                .speed(0.01),
        );

        ui.label("Integration step");
        ui.add(
            DragValue::new(self.solver.delta_mut())
                .clamp_range(0.001..=f64::MAX)
                .speed(0.001),
        );

        ui.label("Disruption force");
        ui.add(
            DragValue::new(&mut self.disruption_strength)
                .clamp_range(0.0..=f64::MAX)
                .speed(0.25),
        );

        if ui.button("Random disruption").clicked() {
            self.apply_random_disruption();
        }

        ui.label("Point mass");
        let mut point_mass = self.solver.ode().point_mass();

        if ui
            .add(
                DragValue::new(&mut point_mass)
                    .clamp_range(0.01..=100.0)
                    .speed(0.25),
            )
            .changed()
        {
            self.solver.ode_mut().set_point_mass(point_mass);
        }

        ui.label("Elasticity coefficient");
        ui.add(
            DragValue::new(&mut self.solver.ode_mut().elasticity_coefficient)
                .clamp_range(0.0..=1.0)
                .speed(0.01),
        );

        ui.label("Mass connection spring constant");
        ui.add(
            DragValue::new(&mut self.solver.ode_mut().inner_spring_constant)
                .clamp_range(0.00..=100.0)
                .speed(0.05),
        );

        ui.label("Mass-frame connection spring constant");
        ui.add(
            DragValue::new(&mut self.solver.ode_mut().corner_spring_constant)
                .clamp_range(0.00..=100.0)
                .speed(0.05),
        );

        ui.label("Damping factor");
        ui.add(
            DragValue::new(&mut self.solver.ode_mut().damping_factor)
                .clamp_range(0.0..=100.0)
                .speed(0.05),
        );
    }
}

pub struct Jelly {
    camera: Camera,

    bezier_cube: BezierCube,
    bezier_patches: BezierPatches,
    model: Model,
    room: Room,
    control_frame: ControlFrame,
    simulation: Simulation,
}

impl Jelly {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let control_frame_transform = Rc::new(RefCell::new(jelly::ControlFrameTransform::new()));
        let bezier_cube = BezierCube::new(Arc::clone(&gl));

        Self {
            camera: Camera::new(),

            bezier_patches: BezierPatches::new(Arc::clone(&gl), &bezier_cube.cube),
            bezier_cube,
            model: Model::new(Arc::clone(&gl)),
            room: Room::new(Arc::clone(&gl)),
            control_frame: ControlFrame::new(Arc::clone(&gl), Rc::clone(&control_frame_transform)),
            simulation: Simulation::new(control_frame_transform),
        }
    }
}

impl Presenter for Jelly {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        self.bezier_cube.ui(ui);
        self.model.ui(ui);
        self.bezier_patches.ui(ui);
        self.room.ui(ui);
        ui.separator();
        self.control_frame.ui(ui);
        ui.separator();
        self.simulation.ui(ui);
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
        self.bezier_patches.draw(aspect_ratio, &self.camera);
        self.control_frame.draw(aspect_ratio, &self.camera);
        self.room.draw(aspect_ratio, &self.camera);
    }

    fn update(&mut self, delta: std::time::Duration) {
        self.simulation
            .update(&mut self.bezier_cube, &mut self.bezier_patches, delta);
    }

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
