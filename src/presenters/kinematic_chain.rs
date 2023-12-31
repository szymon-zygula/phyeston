use super::{Presenter, PresenterBuilder};
use crate::controls::mouse::MouseState;
use crate::numerics::{kinematics::flat_chain, Rect};
use crate::render::{
    gl_drawable::GlDrawable,
    gl_mesh::{GlLines, GlTriangleMesh},
    gl_program::GlProgram,
    gl_texture::GlTexture,
    models,
};
use crate::simulators::kinematic_chain::*;
use egui::{widgets::DragValue, Ui};
use egui_winit::winit::dpi::PhysicalSize;
use glow::HasContext;
use nalgebra as na;
use std::sync::Arc;

#[derive(Debug)]
enum DrawingRectState {
    Drawing(Rect),
    NotDrawing,
}

pub struct KinematicChain {
    rect_program: GlProgram,
    texture_program: GlProgram,
    rect_mesh: GlTriangleMesh,

    drawing_rect: DrawingRectState,
    rects: Vec<Rect>,

    config_state_start: flat_chain::ReverseSolutions,
    start: na::Point2<f64>,
    start_arm_mesh: GlLines,

    current_arm_mesh: GlLines,
    current_path: Option<Vec<na::Point2<f64>>>,

    config_state_end: flat_chain::ReverseSolutions,
    end: na::Point2<f64>,
    end_arm_mesh: GlLines,

    config_obstruction: ConfigObstuction,
    texture: GlTexture,
    map: BFSMap,
    system: flat_chain::System,

    start_with_second: bool,
    end_with_second: bool,

    simulation_speed: f64,
    animation_progress: f64,

    gl: Arc<glow::Context>,
}

impl KinematicChain {
    const ARM_ORIGIN: na::Point2<f64> = na::point![1000.0, 500.0];

    fn new(gl: Arc<glow::Context>) -> Self {
        let system = flat_chain::System::new(100.0, 100.0);
        let config_obstuction = ConfigObstuction::new(system, Self::ARM_ORIGIN);
        let map = BFSMap::from_obstructions(&Some(na::point![0.0, 0.0]), &config_obstuction);
        let texture = config_obstuction.texture(&map, None);

        let mut me = Self {
            rect_program: GlProgram::vertex_fragment(Arc::clone(&gl), "2d_vert", "pass_frag"),
            texture_program: GlProgram::vertex_fragment(
                Arc::clone(&gl),
                "texture_vert",
                "texture_frag",
            ),
            rect_mesh: GlTriangleMesh::new(Arc::clone(&gl), &models::rect()),

            drawing_rect: DrawingRectState::NotDrawing,
            rects: Vec::new(),

            config_state_start: flat_chain::ReverseSolutions::One(na::Point2::origin()),
            start: Self::ARM_ORIGIN + na::vector![200.0, 0.0],
            start_arm_mesh: GlLines::new(Arc::clone(&gl), &[na::Point::origin(); 8]),

            current_path: None,
            current_arm_mesh: GlLines::new(Arc::clone(&gl), &[na::Point::origin(); 4]),

            config_state_end: flat_chain::ReverseSolutions::One(na::Point2::origin()),
            end: Self::ARM_ORIGIN + na::vector![200.0, 0.0],
            end_arm_mesh: GlLines::new(Arc::clone(&gl), &[na::Point::origin(); 8]),

            config_obstruction: config_obstuction,
            texture: GlTexture::new(Arc::clone(&gl), &texture),
            map,
            system,

            start_with_second: false,
            end_with_second: false,

            simulation_speed: 100.0,
            animation_progress: 0.0,

            gl,
        };

        me.update_arm_mesh();

        me
    }

    fn reset_all(&mut self) {
        self.config_state_start = self.config_obstruction.correct_solution(
            &self
                .system
                .inverse_kinematics(&(self.start - Self::ARM_ORIGIN).into()),
        );
        self.config_state_end = self.config_obstruction.correct_solution(
            &self
                .system
                .inverse_kinematics(&(self.end - Self::ARM_ORIGIN).into()),
        );

        self.update_map();
        self.update_arm_mesh();
    }

    fn update_obstruction_texture(&mut self) {
        self.texture = GlTexture::new(
            Arc::clone(&self.gl),
            &self
                .config_obstruction
                .texture(&self.map, self.current_path.as_ref().map(|v| v.as_slice())),
        );
    }

    fn update_arm_mesh(&mut self) {
        self.start_arm_mesh
            .update_points(&self.arm_points(&self.config_state_start));
        self.end_arm_mesh
            .update_points(&self.arm_points(&self.config_state_end));
    }

    fn arm_points(&self, config: &flat_chain::ReverseSolutions) -> Vec<na::Point3<f32>> {
        let origin = na::point![Self::ARM_ORIGIN.x as f32, Self::ARM_ORIGIN.y as f32, 0.0];
        match config {
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
        }
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

    fn reset_obstruction(&mut self) {
        self.config_obstruction = ConfigObstuction::new(self.system, Self::ARM_ORIGIN);

        for rect in &self.rects {
            self.config_obstruction.add_rect(rect);
        }

        self.update_map();
    }

    fn draw_arm(&self, size: Option<PhysicalSize<u32>>) {
        let Some(size) = size else { return };

        self.rect_program.enable();
        self.rect_program
            .uniform_matrix_4_f32_slice("view_transform", Self::view_matrix(size).as_slice());
        self.rect_program
            .uniform_matrix_4_f32_slice("model_transform", na::Matrix4::identity().as_slice());

        match self.config_state_start {
            flat_chain::ReverseSolutions::Two(_, _) | flat_chain::ReverseSolutions::One(_) => {
                self.start_arm_mesh.draw();
            }
            flat_chain::ReverseSolutions::None | flat_chain::ReverseSolutions::InfinitelyMany => {}
        }

        if self.current_path.is_some() {
            self.current_arm_mesh.draw();
        }

        match self.config_state_end {
            flat_chain::ReverseSolutions::Two(_, _) | flat_chain::ReverseSolutions::One(_) => {
                self.end_arm_mesh.draw();
            }
            flat_chain::ReverseSolutions::None | flat_chain::ReverseSolutions::InfinitelyMany => {}
        }
    }

    fn draw_texture(&self, size: Option<PhysicalSize<u32>>) {
        let Some(size) = size else { return };
        let aspect_ratio = size.width as f32 / size.height as f32;

        self.texture_program.enable();
        self.texture_program.uniform_matrix_4_f32_slice(
            "view_transform",
            na::matrix![
                1.0 / aspect_ratio, 0.0, 0.0, 0.0;
                0.0, 1.0, 0.0, 0.0;
                0.0, 0.0, 1.0, 0.0;
                0.0, 0.0, 0.0, 1.0;
            ]
            .as_slice(),
        );
        self.texture_program.uniform_matrix_4_f32_slice(
            "model_transform",
            na::Translation3::new(aspect_ratio * 1.0 - 0.25, 0.75, -1.0)
                .to_homogeneous()
                .as_slice(),
        );
        self.texture.bind_to_image_unit(0);
        self.rect_mesh.draw();
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

    fn update_map(&mut self) {
        let start = match self.config_state_start {
            flat_chain::ReverseSolutions::InfinitelyMany => None,
            flat_chain::ReverseSolutions::Two(first, second) => Some(if self.start_with_second {
                second
            } else {
                first
            }),
            flat_chain::ReverseSolutions::One(sol) => Some(sol),
            flat_chain::ReverseSolutions::None => None,
        };

        self.map = BFSMap::from_obstructions(&start, &self.config_obstruction);

        self.update_path();
        self.update_obstruction_texture();
    }

    fn update_path(&mut self) {
        self.animation_progress = 0.0;
        self.current_path = match self.config_state_end {
            flat_chain::ReverseSolutions::InfinitelyMany => None,
            flat_chain::ReverseSolutions::Two(t_1, t_2) => self
                .map
                .path_to(if self.end_with_second { &t_2 } else { &t_1 }),
            flat_chain::ReverseSolutions::One(target) => self.map.path_to(&target),
            flat_chain::ReverseSolutions::None => None,
        }
    }

    fn handle_rect_setting(&mut self, state: &MouseState) {
        if state.is_middle_button_down() {
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
                self.config_obstruction.add_rect(rect);
                self.rects.push(*rect);
                self.drawing_rect = DrawingRectState::NotDrawing;
                self.reset_all();
            }
        }
    }

    fn handle_target_setting(&mut self, state: &MouseState) {
        let Some(position) = state.position() else {
            return;
        };

        if state.is_left_button_down() {
            self.start = na::point![position.x, position.y];
            self.config_state_start = self.config_obstruction.correct_solution(
                &self
                    .system
                    .inverse_kinematics(&(self.start - Self::ARM_ORIGIN).into()),
            );
            self.reset_all();
        }

        if state.is_right_button_down() {
            self.end = na::point![position.x, position.y];
            self.config_state_end = self.config_obstruction.correct_solution(
                &self
                    .system
                    .inverse_kinematics(&(self.end - Self::ARM_ORIGIN).into()),
            );
            self.reset_all();
        }
    }

    fn update_current_mesh(&mut self, frame: usize) {
        let Some(path) = &self.current_path else {
            return;
        };

        self.current_arm_mesh
            .update_points(&self.state_to_points(&path[frame]));
    }
}

impl Presenter for KinematicChain {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.add(DragValue::new(&mut self.simulation_speed).clamp_range(0.0..=1000.0));
        ui.label(format!(
            "Animation progress: {:.2}",
            self.animation_progress
        ));

        let mut reset = false;

        reset |= ui
            .add_enabled(
                matches!(
                    self.config_state_start,
                    flat_chain::ReverseSolutions::Two(_, _)
                ),
                egui::Checkbox::new(&mut self.start_with_second, "Start with second solution"),
            )
            .changed();

        reset |= ui
            .add_enabled(
                matches!(
                    self.config_state_end,
                    flat_chain::ReverseSolutions::Two(_, _)
                ),
                egui::Checkbox::new(&mut self.end_with_second, "End with second solution"),
            )
            .changed();

        if reset {
            self.reset_all();
        }

        if (ui.label("l_1")
            | ui.add(DragValue::new(&mut self.system.l_1).clamp_range(0.0..=300.0))
            | ui.label("l_2")
            | ui.add(DragValue::new(&mut self.system.l_2).clamp_range(0.0..=300.0)))
        .changed()
        {
            self.reset_obstruction();

            self.config_state_start = self
                .system
                .inverse_kinematics(&(self.start - Self::ARM_ORIGIN).into());
            self.config_state_end = self
                .system
                .inverse_kinematics(&(self.end - Self::ARM_ORIGIN).into());

            self.reset_all();
        }

        ui.label("Rects");
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut change = false;
            self.rects.retain_mut(|rect| {
                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        change |= ui
                            .add(
                                DragValue::new(&mut rect.p_1.x)
                                    .speed(1.0)
                                    .clamp_range(0.0..=2000.0),
                            )
                            .changed();
                        ui.label("x");
                        change |= ui
                            .add(
                                DragValue::new(&mut rect.p_1.y)
                                    .speed(1.0)
                                    .clamp_range(0.0..=2000.0),
                            )
                            .changed();
                        ui.label(",");
                        change |= ui
                            .add(
                                DragValue::new(&mut rect.p_2.x)
                                    .speed(1.0)
                                    .clamp_range(0.0..=2000.0),
                            )
                            .changed();
                        ui.label("x");
                        change |= ui
                            .add(
                                DragValue::new(&mut rect.p_2.y)
                                    .speed(1.0)
                                    .clamp_range(0.0..=2000.0),
                            )
                            .changed();
                    });

                    let stays = !ui.button("X").clicked();
                    change |= !stays;
                    stays
                })
                .inner
            });

            if change {
                self.reset_obstruction();
            }
        });
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.label("Bottom text");
    }

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        self.draw_rects(size);
        self.draw_arm(size);
        self.draw_texture(size);
    }

    fn update(&mut self, delta: std::time::Duration) {
        let Some(path) = &self.current_path else {
            return;
        };

        let animation_progress_old = self.animation_progress;
        self.animation_progress = (self.animation_progress
            + delta.as_secs_f64() * self.simulation_speed)
            .rem_euclid(path.len() as f64);

        if animation_progress_old.floor() != self.animation_progress.floor() {
            self.update_current_mesh(self.animation_progress.floor() as usize);
        }
    }

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
