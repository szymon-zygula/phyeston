use super::{Presenter, PresenterBuilder};
use crate::{
    controls::{camera::Camera, mouse::MouseState},
    numerics::rotations::*,
    render::{
        drawbuffer::Drawbuffer, gl_drawable::GlDrawable, gl_mesh::GlTriangleMesh,
        gl_program::GlProgram, models,
    },
    ui::widgets,
};
use egui::{widgets::DragValue, Ui};
use egui_winit::winit::dpi::PhysicalSize;
use na::SimdPartialOrd;
use nalgebra as na;
use std::cell::RefCell;
use std::sync::Arc;

pub struct Quaternions {
    camera: Camera,

    drawbuffer: RefCell<Option<Drawbuffer>>,
    meshes_program: GlProgram,
    cube_mesh: GlTriangleMesh,
    gl: Arc<glow::Context>,

    start_rotation_euler: EulerAngles,
    start_rotation_quaternion: Quaternion,
    start_position: na::Vector3<f64>,

    end_rotation_euler: EulerAngles,
    end_rotation_quaternion: Quaternion,
    end_position: na::Vector3<f64>,

    slerp: bool,

    animation_time: f64,

    keyframes_quaternion: Vec<na::Matrix4<f32>>,
    keyframes_euler: Vec<na::Matrix4<f32>>,

    current_time: f64,
    current_quaternion: na::Matrix4<f32>,
    current_euler: na::Matrix4<f32>,
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
        slerp: bool,
        keyframes: usize,
    ) -> Self {
        let start_rotation_euler = start_rotation.normalize().to_euler_angles().normalize();
        let start_rotation_quaternion = start_rotation.normalize().to_quaternion().normalize();
        let end_rotation_euler = end_rotation.normalize().to_euler_angles().normalize();
        let end_rotation_quaternion = end_rotation.normalize().to_quaternion().normalize();

        let keyframes_euler = Self::euler_keyframes(
            &start_rotation_euler,
            &start_position,
            &end_rotation_euler,
            &end_position,
            keyframes,
        );

        let keyframes_quaternion = Self::quaternion_keyframes(
            &start_rotation_quaternion,
            &start_position,
            &end_rotation_quaternion,
            &end_position,
            keyframes,
            slerp,
        );

        Self {
            camera: Camera::new(),

            drawbuffer: RefCell::new(None),
            meshes_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "perspective_vert",
                "phong_frag",
            ),
            cube_mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::cube()),
            gl,

            animation_time: 5.0,

            start_rotation_euler,
            start_rotation_quaternion,
            start_position,
            end_rotation_euler,
            end_rotation_quaternion,
            end_position,

            slerp,

            current_time: 0.0,
            current_quaternion: keyframes_quaternion[0],
            current_euler: keyframes_euler[0],

            keyframes_euler,
            keyframes_quaternion,
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

    fn euler_keyframe(
        start_euler: &EulerAngles,
        start_position: &na::Vector3<f64>,
        end_euler: &EulerAngles,
        end_position: &na::Vector3<f64>,
        t: f64,
    ) -> na::Matrix4<f32> {
        na::Translation::from(na::Vector3::lerp(start_position, end_position, t))
            .to_homogeneous()
            .map(|r| r as f32)
            * EulerAngles::lerp(&start_euler, &end_euler, t)
                .to_homogeneous()
                .map(|r| r as f32)
    }

    fn euler_keyframes(
        start_euler: &EulerAngles,
        start_position: &na::Vector3<f64>,
        end_euler: &EulerAngles,
        end_position: &na::Vector3<f64>,
        keyframes: usize,
    ) -> Vec<na::Matrix4<f32>> {
        (0..=keyframes + 1)
            .map(|i| {
                let t = (i as f64) / (keyframes as f64 + 1.0);
                Self::euler_keyframe(&start_euler, start_position, &end_euler, end_position, t)
            })
            .collect()
    }

    fn quaternion_keyframe(
        interpolation: fn(&Quaternion, &Quaternion, f64) -> Quaternion,
        start_rotation: &Quaternion,
        start_position: &na::Vector3<f64>,
        end_rotation: &Quaternion,
        end_position: &na::Vector3<f64>,
        t: f64,
    ) -> na::Matrix4<f32> {
        na::Translation::from(na::Vector3::lerp(start_position, end_position, t))
            .to_homogeneous()
            .map(|r| r as f32)
            * interpolation(&start_rotation, &end_rotation, t)
                .to_homogeneous()
                .map(|r| r as f32)
    }

    fn quaternion_keyframes(
        start_quaternion: &Quaternion,
        start_position: &na::Vector3<f64>,
        end_quaternion: &Quaternion,
        end_position: &na::Vector3<f64>,
        keyframes: usize,
        slerp: bool,
    ) -> Vec<na::Matrix4<f32>> {
        let interpolation = if slerp {
            Quaternion::slerp
        } else {
            Quaternion::lerp
        };

        (0..=keyframes + 1)
            .map(|i| {
                let t = (i as f64) / (keyframes as f64 + 1.0);
                Self::quaternion_keyframe(
                    interpolation,
                    start_quaternion,
                    start_position,
                    end_quaternion,
                    end_position,
                    t,
                )
            })
            .collect()
    }

    fn draw_axis(
        &self,
        keyframes: &[na::Matrix4<f32>],
        current_frame: &na::Matrix4<f32>,
        vector: &na::Vector3<f32>,
        color: &[f32; 4],
    ) {
        let ones = na::vector![1.0, 1.0, 1.0];
        let scale = 0.6 * (ones * 0.1 + vector).simd_clamp(na::Vector3::zeros(), ones);
        let translation = 0.4 * vector;

        let base_transform = na::Translation3::from(translation).to_homogeneous()
            * na::Scale3::from(scale).to_homogeneous();
        self.meshes_program
            .uniform_4_f32_slice("material_color", color);

        for transform in keyframes {
            self.meshes_program.uniform_matrix_4_f32_slice(
                "model_transform",
                (transform * base_transform).as_slice(),
            );
            self.cube_mesh.draw();
        }

        self.meshes_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (current_frame * base_transform).as_slice(),
        );
        self.cube_mesh.draw();
    }

    fn draw_axes(&self, current_frame: &na::Matrix4<f32>, keyframes: &[na::Matrix4<f32>]) {
        self.meshes_program.uniform_f32("material_diffuse", 0.8);
        self.meshes_program.uniform_f32("material_specular", 0.4);
        self.meshes_program
            .uniform_f32("material_specular_exp", 10.0);

        self.draw_axis(
            keyframes,
            current_frame,
            &na::vector![1.0, 0.0, 0.0],
            &[1.0, 0.0, 0.0, 1.0],
        );
        self.draw_axis(
            keyframes,
            current_frame,
            &na::vector![0.0, 1.0, 0.0],
            &[0.0, 1.0, 0.0, 1.0],
        );
        self.draw_axis(
            keyframes,
            current_frame,
            &na::vector![0.0, 0.0, 1.0],
            &[0.0, 0.0, 1.0, 1.0],
        );
    }

    fn draw_meshes(&self, size: PhysicalSize<u32>) {
        let aspect_ratio = 0.5 * size.width as f32 / size.height as f32;
        let drawbuffer = self.drawbuffer.borrow();
        let Some(drawbuffer) = drawbuffer.as_ref() else {
            return;
        };

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

        drawbuffer.clear();
        drawbuffer.draw_with(|| {
            self.draw_axes(&self.current_euler, &self.keyframes_euler);
        });
        drawbuffer.blit(0, 0);

        drawbuffer.clear();
        drawbuffer.draw_with(|| {
            self.draw_axes(&self.current_quaternion, &self.keyframes_quaternion);
        });
        drawbuffer.blit(drawbuffer.size().width, 0);
    }
}

impl Presenter for Quaternions {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.label("Animation time");
        ui.add(
            DragValue::new(&mut self.animation_time)
                .clamp_range(0.0..=20.0)
                .speed(0.5),
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
        self.current_time += delta.as_secs_f64() / self.animation_time;
        self.current_time = self.current_time.clamp(0.0, 1.0);

        let interpolation = if self.slerp {
            Quaternion::slerp
        } else {
            Quaternion::lerp
        };

        self.current_euler = Self::euler_keyframe(
            &self.start_rotation_euler,
            &self.start_position,
            &self.end_rotation_euler,
            &self.end_position,
            self.current_time,
        );

        self.current_quaternion = Self::quaternion_keyframe(
            interpolation,
            &self.start_rotation_quaternion,
            &self.start_position,
            &self.end_rotation_quaternion,
            &self.end_position,
            self.current_time,
        );
    }

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
    slerp: bool,
    keyframes: usize,
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

                    widgets::vector_drag(ui, vector, 0.0, 360.0, "°", 1.0, &["z", "y", "x"])
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
            | ui.checkbox(&mut self.slerp, "Use spherical quaternion interpolation")
            | ui.add(DragValue::new(&mut self.keyframes).clamp_range(0..=100))
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(Quaternions::new(
            gl,
            self.start_rotation,
            self.start_position,
            self.end_rotation,
            self.end_position,
            self.slerp,
            self.keyframes,
        ))
    }
}
