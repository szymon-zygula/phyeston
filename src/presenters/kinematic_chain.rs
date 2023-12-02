use super::{Presenter, PresenterBuilder};
use crate::{
    controls::mouse::MouseState,
    numerics::kinematics::flat_chain,
    render::{
        gl_drawable::GlDrawable,
        gl_mesh::{GlLines, GlTriangleMesh},
        gl_program::GlProgram,
        models,
    },
};
use egui::{widgets::DragValue, Ui};
use egui_winit::winit::dpi::PhysicalSize;
use glow::HasContext;
use nalgebra as na;
use std::sync::Arc;

const ORIGIN: na::Point2<f32> = na::point![800.0, 500.0];

#[derive(Clone, Copy, Debug)]
struct Rect {
    p_1: na::Point2<f64>,
    p_2: na::Point2<f64>,
}

#[derive(Debug)]
enum DrawingRectState {
    Drawing(Rect),
    NotDrawing,
}

pub struct KinematicChain {
    rect_program: GlProgram,
    rect_mesh: GlTriangleMesh,
    arm_mesh: GlLines,

    drawing_rect: DrawingRectState,
    rects: Vec<Rect>,

    system: flat_chain::System,
    config_state: flat_chain::ReverseSolutions,
    target: na::Point2<f64>,

    simulation_speed: f64,

    gl: Arc<glow::Context>,
}

impl KinematicChain {
    const ARM_ORIGIN: na::Point2<f64> = na::point![1000.0, 500.0];

    fn new(gl: Arc<glow::Context>) -> Self {
        let mut me = Self {
            rect_program: GlProgram::vertex_fragment(Arc::clone(&gl), "2d_vert", "pass_frag"),
            rect_mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::rect()),
            arm_mesh: GlLines::new(Arc::clone(&gl), &[na::Point::origin(); 8]),

            drawing_rect: DrawingRectState::NotDrawing,
            rects: Vec::new(),

            system: flat_chain::System::new(100.0, 100.0),
            config_state: flat_chain::ReverseSolutions::One(na::Point2::origin()),
            target: Self::ARM_ORIGIN + na::vector![200.0, 0.0],

            simulation_speed: 1.0,

            gl,
        };

        me.update_arm_mesh();

        me
    }

    fn update_arm_mesh(&mut self) {
        let origin = na::point![Self::ARM_ORIGIN.x as f32, Self::ARM_ORIGIN.y as f32, 0.0];

        self.arm_mesh.update_points(&match self.config_state {
            flat_chain::ReverseSolutions::Two(state_1, state_2) => [
                self.state_to_points(&state_1),
                self.state_to_points(&state_2),
            ]
            .concat(),
            flat_chain::ReverseSolutions::One(state) => {
                [[origin; 4], self.state_to_points(&state)].concat()
            }
            flat_chain::ReverseSolutions::None => vec![origin; 8],
            flat_chain::ReverseSolutions::InfinitelyMany => vec![origin; 8],
        });
    }

    fn state_to_points(&self, state: &na::Point2<f64>) -> [na::Point3<f32>; 4] {
        let origin = na::point![Self::ARM_ORIGIN.x as f32, Self::ARM_ORIGIN.y as f32, 0.0];

        let mut state = self.system.forward_kinematics(state);

        state.p_1 += Self::ARM_ORIGIN.coords;
        state.p_2 += Self::ARM_ORIGIN.coords;
        let p_1 = na::point![state.p_1.x as f32, state.p_1.y as f32, 0.0];
        let p_2 = na::point![state.p_2.x as f32, state.p_2.y as f32, 0.0];

        [origin, p_1, p_1, p_2]
    }

    fn view_matrix(size: PhysicalSize<u32>) -> na::Matrix4<f32> {
        let width = size.width as f32;
        let height = size.height as f32;

        na::matrix![
            2.0 / width, 0.0, 0.0, -1.0;
            0.0, -2.0 / height, 0.0, 1.0;
            0.0, 0.0, 1.0, 0.0;
            0.0, 0.0, 0.0, 1.0;
        ]
    }

    fn draw_arm(&self, size: Option<PhysicalSize<u32>>) {
        let Some(size) = size else { return };

        self.rect_program.enable();
        self.rect_program
            .uniform_matrix_4_f32_slice("view_transform", Self::view_matrix(size).as_slice());
        self.rect_program
            .uniform_matrix_4_f32_slice("model_transform", na::Matrix4::identity().as_slice());
        self.arm_mesh.draw();
    }

    fn draw_rects(&self, size: Option<PhysicalSize<u32>>) {
        let Some(size) = size else { return };

        self.rect_program.enable();
        self.rect_program
            .uniform_matrix_4_f32_slice("view_transform", Self::view_matrix(size).as_slice());

        unsafe { self.gl.disable(glow::CULL_FACE) };

        for rect in &self.rects {
            self.draw_rect(rect);
        }

        if let DrawingRectState::Drawing(rect) = self.drawing_rect {
            self.draw_rect(&rect);
        }

        unsafe { self.gl.enable(glow::CULL_FACE) };
    }

    fn draw_rect(&self, rect: &Rect) {
        let lengths = (rect.p_1 - rect.p_2).map(|c| c.abs() as f32);
        let center = 0.5 * (rect.p_1.coords + rect.p_2.coords).map(|c| c as f32);

        self.rect_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (na::geometry::Translation3::new(center.x, center.y, 0.0).to_homogeneous()
                * na::geometry::Scale3::new(2.0 * lengths.x, 2.0 * lengths.y, 0.0)
                    .to_homogeneous())
            .as_slice(),
        );
        self.rect_mesh.draw();
    }

    fn handle_rect_setting(&mut self, state: &MouseState) {
        if state.is_right_button_down() {
            if let Some(position) = state.position() {
                let current_point = na::point![position.x, position.y];
                self.drawing_rect = match self.drawing_rect {
                    DrawingRectState::Drawing(Rect { p_1, .. }) => {
                        DrawingRectState::Drawing(Rect {
                            p_1,
                            p_2: current_point,
                        })
                    }
                    DrawingRectState::NotDrawing => DrawingRectState::Drawing(Rect {
                        p_1: current_point,
                        p_2: current_point,
                    }),
                };
            }
        } else {
            if let DrawingRectState::Drawing(rect) = &self.drawing_rect {
                self.rects.push(*rect);
                self.drawing_rect = DrawingRectState::NotDrawing;
            }
        }
    }

    fn handle_target_setting(&mut self, state: &MouseState) {
        if !state.is_left_button_down() {
            return;
        }

        let Some(position) = state.position() else {
            return;
        };

        self.target = na::point![position.x, position.y];
        self.config_state = self
            .system
            .inverse_kinematics(&(self.target - Self::ARM_ORIGIN).into());
        self.update_arm_mesh();
    }
}

impl Presenter for KinematicChain {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.add(DragValue::new(&mut self.simulation_speed).clamp_range(0.0..=100.0));

        if (ui.add(DragValue::new(&mut self.system.l_1).clamp_range(0.0..=300.0))
            | ui.add(DragValue::new(&mut self.system.l_2).clamp_range(0.0..=300.0)))
        .changed()
        {
            self.config_state = self
                .system
                .inverse_kinematics(&(self.target - Self::ARM_ORIGIN).into());
            self.update_arm_mesh();
        }

        ui.label("Rects");
        egui::ScrollArea::vertical().show(ui, |ui| {
            for rect in &self.rects {
                ui.label(format!(
                    "{:.3}x{:.3}, {:.3}x{:.3}",
                    rect.p_1.x, rect.p_1.y, rect.p_2.x, rect.p_2.y
                ));
            }
        });
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        self.draw_rects(size);
        self.draw_arm(size);
    }

    fn update(&mut self, delta: std::time::Duration) {}

    fn update_mouse(&mut self, state: MouseState) {
        self.handle_rect_setting(&state);
        self.handle_target_setting(&state);
    }

    fn name(&self) -> &'static str {
        "Kinematic chain"
    }
}

#[derive(Default)]
pub struct KinematicChainBuilder {}

impl KinematicChainBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for KinematicChainBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("")
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(KinematicChain::new(gl))
    }
}
