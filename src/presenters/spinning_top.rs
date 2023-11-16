use super::{Presenter, PresenterBuilder};
use crate::{
    controls::{camera::Camera, mouse::MouseState},
    numerics::{
        ode::{self, Solver},
        RungeKuttaIV,
    },
    render::{
        gl_drawable::GlDrawable,
        gl_program::GlProgram,
        mesh::{GlLineStrip, GlTriangleMesh},
        models,
    },
    simulators::spinning_top::SpinningTopODE,
};
use egui::{widgets::DragValue, Ui};
use glow::HasContext;
use nalgebra as na;
use std::sync::Arc;

pub struct SpinningTop {
    meshes_program: GlProgram,
    box_mesh: GlTriangleMesh,
    plane_mesh: GlTriangleMesh,

    strips_program: GlProgram,
    gravity_strip: GlLineStrip,
    trajectory_strip: GlLineStrip,
    diagonal_strip: GlLineStrip,

    camera: Camera,

    state: ode::State<7>,
    solver: RungeKuttaIV<7, SpinningTopODE>,
    simulation_speed: f64,
    exact_t: f64,

    show_trajectory: bool,
    show_plane: bool,
    show_gravity_vector: bool,
    show_box: bool,
    show_diagonal: bool,

    max_trajectory_points: usize,

    gl: Arc<glow::Context>,
}

impl SpinningTop {
    const LIGHT_POSITION: na::Vector3<f32> = na::vector![-2.0, 4.0, -2.0];
    const LIGHT_COLOR: na::Vector3<f32> = na::vector![2.0, 2.0, 2.0];
    const LIGHT_AMBIENT: na::Vector3<f32> = na::vector![0.4, 0.4, 0.4];
    const PLANE_SCALE: f32 = 3.0;

    const BOX_COLOR: na::Vector4<f32> = na::vector![0.2, 0.4, 0.8, 0.7];
    const PLANE_COLOR: na::Vector4<f32> = na::vector![0.8, 0.4, 0.2, 0.4];

    const DEFAULT_DENSITY: f64 = 10.0;
    const DEFAULT_SIDE_LENGTH: f64 = 2.0;
    const DEFAULT_MAX_TRAJECTORY_POINTS: usize = 10000;
    const MAX_TRAJECTORY_POINTS_LIMIT: usize = 1024 * 1024;

    pub fn new(
        gl: Arc<glow::Context>,
        rotation: na::UnitQuaternion<f64>,
        angular_velocity: na::Vector3<f64>,
    ) -> Self {
        let mut state = ode::State::<7> {
            t: 0.0,
            y: na::SVector::<f64, 7>::zeros(),
        };

        state.y[0] = angular_velocity.x;
        state.y[1] = angular_velocity.y;
        state.y[2] = angular_velocity.z;
        state.y[3] = rotation.w;
        state.y[4] = rotation.i;
        state.y[5] = rotation.j;
        state.y[6] = rotation.k;

        Self {
            meshes_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "perspective_vert",
                "phong_frag",
            ),
            box_mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::cube()),
            plane_mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::double_plane()),

            strips_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "perspective_vert",
                "color_frag",
            ),
            gravity_strip: GlLineStrip::new(
                Arc::clone(&gl),
                &[na::point![0.0, 0.0, 0.0], na::point![0.0, -1.0, 0.0]],
            ),
            trajectory_strip: GlLineStrip::with_capacity(
                Arc::clone(&gl),
                Self::DEFAULT_MAX_TRAJECTORY_POINTS,
            ),
            diagonal_strip: Self::diagonal_strip(Arc::clone(&gl)),

            camera: Camera::new(),

            exact_t: 0.0,
            state,
            solver: RungeKuttaIV::new(
                0.01,
                SpinningTopODE::new(Self::DEFAULT_DENSITY, Self::DEFAULT_SIDE_LENGTH),
            ),
            simulation_speed: 1.0,

            show_box: true,
            show_plane: true,
            show_gravity_vector: false,
            show_trajectory: false,
            show_diagonal: false,

            max_trajectory_points: Self::DEFAULT_MAX_TRAJECTORY_POINTS,

            gl,
        }
    }

    fn set_side_length(&mut self, side_length: f64) {
        self.solver.ode_mut().set_side_length(side_length);
    }

    fn diagonal_strip(gl: Arc<glow::Context>) -> GlLineStrip {
        GlLineStrip::new(
            Arc::clone(&gl),
            &[na::point![-1.0, -1.0, -1.0], na::point![1.0, 1.0, 1.0]],
        )
    }

    fn box_transform(&self) -> na::Matrix4<f32> {
        let rotation = na::UnitQuaternion::new_normalize(na::Quaternion::new(
            self.state.y[3] as f32,
            self.state.y[4] as f32,
            self.state.y[5] as f32,
            self.state.y[6] as f32,
        ));

        let half_side_length = self.solver.ode().side_length() as f32 * 0.5;

        let translation =
            na::Translation3::new(half_side_length, half_side_length, half_side_length);
        rotation.to_homogeneous()
            * translation.to_homogeneous()
            * na::Scale3::new(half_side_length, half_side_length, half_side_length).to_homogeneous()
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

        if self.show_box {
            self.draw_box();
        }

        if self.show_plane {
            self.draw_plane();
        }
    }

    fn draw_box(&self) {
        self.meshes_program
            .uniform_4_f32_slice("material_color", Self::BOX_COLOR.as_slice());
        self.meshes_program.uniform_f32("material_diffuse", 0.8);
        self.meshes_program.uniform_f32("material_specular", 0.4);
        self.meshes_program
            .uniform_f32("material_specular_exp", 10.0);

        self.meshes_program
            .uniform_matrix_4_f32_slice("model_transform", self.box_transform().as_slice());

        self.box_mesh.draw();
    }

    fn draw_plane(&self) {
        self.meshes_program
            .uniform_4_f32_slice("material_color", Self::PLANE_COLOR.as_slice());
        self.meshes_program.uniform_f32("material_diffuse", 0.4);
        self.meshes_program.uniform_f32("material_specular", 0.2);
        self.meshes_program
            .uniform_f32("material_specular_exp", 50.0);

        self.meshes_program.uniform_matrix_4_f32_slice(
            "model_transform",
            na::Scale3::new(Self::PLANE_SCALE, Self::PLANE_SCALE, Self::PLANE_SCALE)
                .to_homogeneous()
                .as_slice(),
        );

        self.plane_mesh.draw();
    }

    fn draw_strips(&self, aspect_ratio: f32) {
        self.strips_program.enable();
        self.strips_program
            .uniform_matrix_4_f32_slice("view_transform", self.camera.view_transform().as_slice());
        self.strips_program.uniform_matrix_4_f32_slice(
            "projection_transform",
            self.camera.projection_transform(aspect_ratio).as_slice(),
        );
        self.strips_program
            .uniform_matrix_4_f32_slice("model_transform", na::Matrix4::identity().as_slice());

        if self.show_gravity_vector {
            self.draw_gravity_vector();
        }

        if self.show_trajectory {
            self.draw_trajectory();
        }

        if self.show_diagonal {
            self.draw_diagonal();
        }
    }

    fn draw_gravity_vector(&self) {
        self.strips_program
            .uniform_4_f32("color", 1.0, 1.0, 1.0, 1.0);
        self.gravity_strip.draw();
    }

    fn draw_trajectory(&self) {
        self.strips_program
            .uniform_4_f32("color", 1.0, 1.0, 1.0, 1.0);
        self.trajectory_strip.draw();
    }

    fn draw_diagonal(&self) {
        unsafe { self.gl.disable(glow::DEPTH_TEST) };

        self.strips_program
            .uniform_matrix_4_f32_slice("model_transform", self.box_transform().as_slice());
        self.strips_program
            .uniform_4_f32("color", 0.5, 1.0, 0.5, 1.0);
        self.diagonal_strip.draw();

        unsafe { self.gl.enable(glow::DEPTH_TEST) };
    }

    fn step_update(&mut self) {
        let mut new_state = self.solver.step(&self.state);
        let new_rotation = na::UnitQuaternion::new_normalize(na::Quaternion::new(
            new_state.y[3],
            new_state.y[4],
            new_state.y[5],
            new_state.y[6],
        ));

        new_state.y[3] = new_rotation.w;
        new_state.y[4] = new_rotation.i;
        new_state.y[5] = new_rotation.j;
        new_state.y[6] = new_rotation.k;

        self.state = new_state;

        let new_tip = self
            .box_transform()
            .transform_point(&na::point![1.0, 1.0, 1.0]);

        self.trajectory_strip.push_vertex(&new_tip);
    }
}

impl Presenter for SpinningTop {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.solver.ode_mut().enable_gravity, "Gravity");
        ui.add(DragValue::new(&mut self.solver.ode_mut().gravity.y).clamp_range(f64::MIN..=0.0));

        ui.checkbox(&mut self.show_plane, "Show plane");
        ui.checkbox(&mut self.show_gravity_vector, "Show gravity vector");
        ui.checkbox(&mut self.show_box, "Show box");
        ui.checkbox(&mut self.show_trajectory, "Show trajectory");
        ui.checkbox(&mut self.show_diagonal, "Show diagonal");

        ui.label("Maximum trajectory points visible");
        if ui
            .add(
                DragValue::new(&mut self.max_trajectory_points)
                    .clamp_range(2..=Self::MAX_TRAJECTORY_POINTS_LIMIT),
            )
            .changed()
        {
            self.trajectory_strip
                .recapacitate(self.max_trajectory_points);
        }

        let mut density = self.solver.ode().density();
        ui.label("Box density");
        if ui
            .add(DragValue::new(&mut density).clamp_range(0.1..=f32::MAX))
            .changed()
        {
            self.solver.ode.set_density(density);
        }

        let mut side_length = self.solver.ode().side_length();
        ui.label("Side length");
        if ui
            .add(
                DragValue::new(&mut side_length)
                    .clamp_range(0.1..=f64::MAX)
                    .speed(0.01),
            )
            .changed()
        {
            self.set_side_length(side_length);
        }

        ui.label("Simulation speed");
        ui.add(
            DragValue::new(&mut self.simulation_speed)
                .clamp_range(0.0..=f64::MAX)
                .speed(0.01),
        );

        ui.label("Integration step");
        ui.add(
            DragValue::new(&mut self.solver.delta)
                .clamp_range(0.001..=f64::MAX)
                .speed(0.001),
        );
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, aspect_ratio: f32) {
        self.draw_meshes(aspect_ratio);
        self.draw_strips(aspect_ratio);
    }

    fn update(&mut self, delta: std::time::Duration) {
        let elapsed_t = delta.as_secs_f64() * self.simulation_speed;
        self.exact_t += elapsed_t;

        while self.exact_t > self.state.t {
            self.step_update();
        }
    }

    fn update_mouse(&mut self, state: MouseState) {
        self.camera.update_from_mouse(state);
    }

    fn name(&self) -> &'static str {
        "Spinning Top"
    }
}

#[derive(Default)]
pub struct SpinningTopBuilder {
    tilt: f64,
    angular_velocity: f64,
}

impl SpinningTopBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for SpinningTopBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("Tilt");
        ui.add(
            DragValue::new(&mut self.tilt)
                .clamp_range(0.0..=180.0)
                .speed(0.1)
                .suffix("°"),
        ) | ui.label("Angular veloctiy")
            | ui.add(
                DragValue::new(&mut self.angular_velocity)
                    .clamp_range(0.0..=f64::MAX)
                    .speed(0.01),
            )
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        let diagonal_angle = std::f64::consts::FRAC_PI_2 - f64::atan2(1.0, f64::sqrt(2.0));
        let axis = na::UnitVector3::new_normalize(na::vector![-1.0, 0.0, 1.0]);
        let rotation =
            na::UnitQuaternion::from_axis_angle(&axis, diagonal_angle + self.tilt.to_radians());

        let angular_velocity = self.angular_velocity
            * na::Rotation3::from_axis_angle(&axis, std::f64::consts::FRAC_PI_2 - diagonal_angle)
                .transform_vector(&na::vector![1.0, 0.0, 1.0]);

        Box::new(SpinningTop::new(gl, rotation, angular_velocity))
    }
}
