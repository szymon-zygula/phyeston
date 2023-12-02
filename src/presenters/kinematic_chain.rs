use super::{Presenter, PresenterBuilder};
use crate::{
    controls::mouse::MouseState,
    numerics::kinematics::flat_chain,
    render::{
        drawbuffer::Drawbuffer, gl_drawable::GlDrawable, gl_mesh::GlTriangleMesh,
        gl_program::GlProgram, models,
    },
    ui::widgets,
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

    drawing_rect: DrawingRectState,
    rects: Vec<Rect>,
    system: flat_chain::System,
    simulation_speed: f64,
    gl: Arc<glow::Context>,
}

impl KinematicChain {
    fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            rect_program: GlProgram::vertex_fragment(Arc::clone(&gl), "2d_vert", "pass_frag"),
            rect_mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::rect()),

            drawing_rect: DrawingRectState::NotDrawing,
            rects: Vec::new(),
            system: flat_chain::System::new(1.0, 1.0),
            simulation_speed: 1.0,
            gl,
        }
    }

    fn draw_rects(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        let Some(size) = size else { return };
        let width = size.width as f32;
        let height = size.height as f32;
        let aspect_ratio = width / height;

        self.rect_program.enable();

        self.rect_program.uniform_matrix_4_f32_slice(
            "view_transform",
            (na::matrix![
                1.0, 0.0, 0.0, 0.0;
                0.0, 1.0, 0.0, 0.0;
                0.0, 0.0, 1.0, 0.0;
                0.0, 0.0, 0.0, 1.0;
            ] * na::matrix![
                2.0 / width, 0.0, 0.0, -1.0;
                0.0, -2.0 / height, 0.0, 1.0;
                0.0, 0.0, 1.0, 0.0;
                0.0, 0.0, 0.0, 1.0;
            ])
            .as_slice(),
        );

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

    fn handle_target_setting(&mut self, state: &MouseState) {}
}

impl Presenter for KinematicChain {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.add(DragValue::new(&mut self.simulation_speed).clamp_range(0.0..=100.0));
        ui.add(DragValue::new(&mut self.system.l_1).clamp_range(0.0..=5.0));
        ui.add(DragValue::new(&mut self.system.l_2).clamp_range(0.0..=5.0));
        ui.add(DragValue::new(&mut self.simulation_speed).clamp_range(0.0..=100.0));

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
