use super::{Presenter, PresenterBuilder};
use crate::{
    controls::{camera::Camera, mouse::MouseState},
    numerics::{angle::Angle, cylinder::Cylinder, rotations::*},
    render::{
        drawbuffer::Drawbuffer, gl_drawable::GlDrawable, gl_mesh::GlTriangleMesh,
        gl_program::GlProgram, gridable::Triangable, mesh::Mesh, models,
    },
    simulators::puma::{ConfigState, CylindersTransforms, Params, SceneState},
    ui::widgets,
};
use egui::{widgets::DragValue, Ui};
use egui_winit::winit::dpi::PhysicalSize;
use na::SimdPartialOrd;
use nalgebra as na;
use std::cell::RefCell;
use std::sync::Arc;

const LIGHT_POSITION: na::Vector3<f32> = na::vector![2.0, 4.0, 2.0];
const LIGHT_COLOR: na::Vector3<f32> = na::vector![2.0, 2.0, 2.0];
const LIGHT_AMBIENT: na::Vector3<f32> = na::vector![0.4, 0.4, 0.4];

struct PumaModel {
    program: GlProgram,
    cylinder: GlTriangleMesh,
    cube: GlTriangleMesh,
    pretransform: na::Matrix4<f32>,
}

impl PumaModel {
    fn new(gl: Arc<glow::Context>) -> Self {
        let (vertices, triangles) = Cylinder::new(1.0, 1.0).triangulation(50, 50);

        Self {
            program: GlProgram::vertex_fragment(Arc::clone(&gl), "perspective_vert", "phong_frag"),
            cylinder: GlTriangleMesh::new(Arc::clone(&gl), &Mesh::new(vertices, triangles)),
            cube: GlTriangleMesh::new(Arc::clone(&gl), &models::cube()),
            pretransform: rotate_x(-std::f64::consts::FRAC_PI_2).map(|c| c as f32),
        }
    }

    fn draw_axis(&self, vector: &na::Vector3<f32>, color: &[f32; 4], transform: &na::Matrix4<f32>) {
        let ones = na::vector![1.0, 1.0, 1.0];
        let scale = 0.6 * (ones * 0.1 + vector).simd_clamp(na::Vector3::zeros(), ones);
        let translation = 0.4 * vector;

        let base_transform = na::Translation3::from(translation).to_homogeneous()
            * na::Scale3::from(scale).to_homogeneous();

        self.program.uniform_4_f32_slice("material_color", color);
        self.program.uniform_matrix_4_f32_slice(
            "model_transform",
            (self.pretransform * transform * base_transform).as_slice(),
        );
        self.cube.draw();
    }

    fn draw_axes(&self, transform: &na::Matrix4<f32>) {
        self.program.uniform_f32("material_diffuse", 0.8);
        self.program.uniform_f32("material_specular", 0.4);
        self.program.uniform_f32("material_specular_exp", 10.0);

        self.draw_axis(
            &na::vector![1.0, 0.0, 0.0],
            &[1.0, 0.0, 0.0, 1.0],
            transform,
        );
        self.draw_axis(
            &na::vector![0.0, 1.0, 0.0],
            &[0.0, 1.0, 0.0, 1.0],
            transform,
        );
        self.draw_axis(
            &na::vector![0.0, 0.0, 1.0],
            &[0.0, 0.0, 1.0, 1.0],
            transform,
        );
    }

    fn draw_puma(&self, transform: &CylindersTransforms) {
        self.program.uniform_f32("material_diffuse", 0.5);
        self.program.uniform_f32("material_specular", 0.8);
        self.program.uniform_f32("material_specular_exp", 20.0);

        self.program
            .uniform_4_f32_slice("material_color", &[1.0, 1.0, 0.0, 1.0]);

        for transform in transform.joint_transforms {
            self.program.uniform_matrix_4_f32_slice(
                "model_transform",
                (self.pretransform * transform.map(|c| c as f32)).as_slice(),
            );
            self.cylinder.draw();
        }

        self.program
            .uniform_4_f32_slice("material_color", &[0.2, 0.2, 0.8, 1.0]);

        for transform in transform.bone_transforms.iter().take(4) {
            self.program.uniform_matrix_4_f32_slice(
                "model_transform",
                (self.pretransform * transform.map(|c| c as f32)).as_slice(),
            );
            self.cylinder.draw();
        }
    }

    fn draw(&self, camera: &Camera, aspect_ratio: f32, transform: &CylindersTransforms) {
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

        self.draw_puma(transform);
        self.draw_axes(&transform.bone_transforms[4].map(|c| c as f32));
    }
}

pub struct Puma {
    puma_model: PumaModel,
    camera: Camera,

    state_right: ConfigState,
    transform_left: CylindersTransforms,
    transform_right: CylindersTransforms,
    params: Params,

    drawbuffer: RefCell<Option<Drawbuffer>>,
    gl: Arc<glow::Context>,

    start_scene: SceneState,
    end_scene: SceneState,

    left_start: ConfigState,
    left_end: ConfigState,
    right_prev: ConfigState,

    animation_time: f64,
    current_time: f64,
    reverse: bool,
}

impl Puma {
    const RIGHT_SAMPLING: f64 = 0.0001;

    fn new(
        gl: Arc<glow::Context>,
        start_scene: SceneState,
        end_scene: SceneState,
        params: Params,
    ) -> Self {
        let start_state = start_scene.inverse_kinematics(&ConfigState::new(), &params);
        let end_state = end_scene.inverse_kinematics(&ConfigState::new(), &params);
        let default_transform = start_state.forward_kinematics(&params);

        Self {
            puma_model: PumaModel::new(Arc::clone(&gl)),
            camera: Camera::new(),

            state_right: start_state,
            transform_left: default_transform.clone(),
            transform_right: default_transform,
            params,

            left_start: start_state,
            left_end: end_state,
            right_prev: ConfigState::new(),

            drawbuffer: RefCell::new(None),
            gl,

            start_scene,
            end_scene,

            animation_time: 2.0,
            current_time: 0.0,
            reverse: false,
        }
    }

    fn drawbuffer_size_matches(&self, size: Option<PhysicalSize<u32>>) -> bool {
        match (size, self.drawbuffer.borrow().as_ref()) {
            (None, None) => true,
            (Some(size), Some(drawbuffer)) => {
                drawbuffer.size().width == size.width as i32 / 2
                    && drawbuffer.size().height == size.height as i32
            }
            _ => false,
        }
    }

    fn recreate_drawbuffer(&self, size: Option<PhysicalSize<u32>>) {
        self.drawbuffer.replace(
            size.map(|s| {
                Drawbuffer::new(Arc::clone(&self.gl), s.width as i32 / 2, s.height as i32)
            }),
        );
    }

    fn draw_meshes(&self, size: PhysicalSize<u32>) {
        let aspect_ratio = 0.5 * size.width as f32 / size.height as f32;
        let drawbuffer = self.drawbuffer.borrow();
        let Some(drawbuffer) = drawbuffer.as_ref() else {
            return;
        };

        drawbuffer.clear();
        drawbuffer.draw_with(|| {
            self.puma_model
                .draw(&self.camera, aspect_ratio, &self.transform_left);
        });
        drawbuffer.blit(0, 0);

        drawbuffer.clear();
        drawbuffer.draw_with(|| {
            self.puma_model
                .draw(&self.camera, aspect_ratio, &self.transform_right);
        });
        drawbuffer.blit(drawbuffer.size().width, 0);
    }
}

fn angle_slider(ui: &mut Ui, text: &str, angle: &mut Angle) -> egui::Response {
    let mut value = angle.deg();
    ui.label(text);

    let response = ui.add(
        DragValue::new(&mut value)
            .clamp_range(0.0..=360.0)
            .speed(0.5),
    );

    if response.changed() {
        angle.set_deg(value);
    }

    response
}

impl Presenter for Puma {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.label("Animation time");
        ui.add(
            DragValue::new(&mut self.animation_time)
                .clamp_range(0.1..=10.0)
                .speed(0.1),
        );
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        if !self.drawbuffer_size_matches(size) {
            self.recreate_drawbuffer(size);
        }

        let Some(size) = size else { return };

        self.draw_meshes(size);
    }

    fn update(&mut self, delta: std::time::Duration) {
        let prev_time = self.current_time;
        self.current_time +=
            if self.reverse { -1.0 } else { 1.0 } * delta.as_secs_f64() / self.animation_time;

        if self.current_time > 1.0 {
            self.current_time = 1.0;
            self.reverse = true;
        } else if self.current_time < 0.0 {
            self.current_time = 0.0;
            self.reverse = false;
        }

        let mut new_right = self
            .start_scene
            .interpolate(&self.end_scene, self.current_time)
            .inverse_kinematics(&self.right_prev, &self.params);

        let mut catchup_time = prev_time;
        while (catchup_time - self.current_time).abs() >= Self::RIGHT_SAMPLING {
            new_right = self
                .start_scene
                .interpolate(&self.end_scene, catchup_time)
                .inverse_kinematics(&new_right, &self.params);

            catchup_time += if catchup_time < self.current_time {
                Self::RIGHT_SAMPLING
            } else {
                -Self::RIGHT_SAMPLING
            };
        }

        self.transform_left = self
            .left_start
            .lerp(&self.left_end, self.current_time)
            .forward_kinematics(&self.params);

        self.transform_right = new_right.forward_kinematics(&self.params);
        self.right_prev = new_right;
    }

    fn update_mouse(&mut self, state: MouseState) {
        self.camera.update_from_mouse(state);
    }

    fn name(&self) -> &'static str {
        "Puma"
    }
}

#[derive(Default)]
pub struct PumaBuilder {
    start_rotation: Rotation,
    start_position: na::Point3<f64>,
    end_rotation: Rotation,
    end_position: na::Point3<f64>,
    keyframes: usize,
    params: Params,
}

impl PumaBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn params_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("l1")
            | ui.add(
                DragValue::new(&mut self.params.l1)
                    .clamp_range(0.0..=5.0)
                    .speed(0.1),
            )
            | ui.label("l3")
            | ui.add(
                DragValue::new(&mut self.params.l3)
                    .clamp_range(0.0..=5.0)
                    .speed(0.1),
            )
            | ui.label("l4")
            | ui.add(
                DragValue::new(&mut self.params.l4)
                    .clamp_range(0.0..=5.0)
                    .speed(0.1),
            )
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

                    widgets::vector_drag(ui, vector, -1.0, 1.0, "", 0.01, &["w", "x", "y", "z"])
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

impl PresenterBuilder for PumaBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        self.params_ui(ui)
            | ui.label("Start frame")
            | Self::frame_ui(
                ui,
                &mut self.start_rotation,
                &mut self.start_position.coords,
            )
            | ui.separator()
            | ui.label("End frame")
            | Self::frame_ui(ui, &mut self.end_rotation, &mut self.end_position.coords)
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        let start_rotation = self.start_rotation.normalize().to_quaternion().normalize();
        let end_rotation = self.end_rotation.normalize().to_quaternion().normalize();

        let start_scene = SceneState::new(self.start_position, start_rotation);
        let end_scene = SceneState::new(self.end_position, end_rotation);

        Box::new(Puma::new(gl, start_scene, end_scene, self.params))
    }
}
