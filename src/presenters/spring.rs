use super::{
    parametrizable_function::{
        constant_function, sine, step_function, step_sine, ConstantFunction,
        ParametrizableFunction, Sine, StepFunction, StepSine,
    },
    Presenter,
};
use crate::{
    numerics::EulerODESolver,
    render::{
        gl_drawable::GlDrawable,
        gl_program::GlProgram,
        mesh::{GlMesh, Mesh, Triangle},
    },
    simulators::spring::{self, SpringODE, SpringState},
};
use egui::{containers::ComboBox, Rgba, Ui};
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints};
use itertools::Itertools;
use nalgebra as na;
use std::{f64::consts::PI, sync::Arc};

macro_rules! state_graph {
    ($states:expr, $field:ident) => {
        $states
            .iter()
            .map(|s| [s.t as f64, s.$field as f64])
            .collect_vec()
    };
}

pub struct Spring {
    gl_program: GlProgram,
    rect_mesh: GlMesh,

    steps_per_frame: usize,
    euler: EulerODESolver<spring::F, 2, SpringODE>,
    states: Vec<SpringState>,
    outer_force_functions: Vec<Box<dyn ParametrizableFunction<F = spring::F>>>,
    selected_outer_force_idx: usize,
    last_clear_t: spring::F,
}

impl Spring {
    pub fn new(gl: Arc<glow::Context>, delta: spring::F, steps_per_frame: usize) -> Self {
        let ode = SpringODE::new(
            1.0,
            Box::new(|_| 1.0),
            2.0,
            0.0,
            1.0,
            0.2,
            Box::new(|_| 0.0),
        );

        Spring {
            rect_mesh: Self::create_rect_mesh(Arc::clone(&gl)),
            gl_program: GlProgram::with_shader_names(
                gl,
                &[
                    ("pass_frag", glow::FRAGMENT_SHADER),
                    ("2d_vert", glow::VERTEX_SHADER),
                ],
            ),
            steps_per_frame,
            euler: EulerODESolver::new(delta, ode),
            states: Vec::new(),
            outer_force_functions: Self::create_outer_force_functions(),
            selected_outer_force_idx: 0,
            last_clear_t: 0.0,
        }
    }

    fn create_outer_force_functions() -> Vec<Box<dyn ParametrizableFunction<F = spring::F>>> {
        let functions: Vec<Box<dyn ParametrizableFunction<F = spring::F>>> = vec![
            Box::new(ConstantFunction::new(
                0.0,
                constant_function::Ranges::new(-5.0..=5.0),
            )),
            Box::new(StepFunction::new(
                1.0,
                0.0,
                step_function::Ranges::new(-5.0..=5.0, -100.0..=100.0),
            )),
            Box::new(StepSine::new(
                1.0,
                1.0,
                0.0,
                step_sine::Ranges::new(-5.0..=5.0, -10.0..=10.0, -PI..=PI),
            )),
            Box::new(Sine::new(
                1.0,
                1.0,
                0.0,
                sine::Ranges::new(-5.0..=5.0, -10.0..=10.0, -PI..=PI),
            )),
        ];

        assert_ne!(functions.len(), 0);
        functions
    }

    fn create_rect_mesh(gl: Arc<glow::Context>) -> GlMesh {
        // 0 1
        // 3 2
        let mesh = Mesh::new(
            vec![
                na::point!(-0.25, 0.25, 0.0),
                na::point!(0.25, 0.25, 0.0),
                na::point!(0.25, -0.25, 0.0),
                na::point!(-0.25, -0.25, 0.0),
            ],
            vec![Triangle([2, 1, 0]), Triangle([3, 2, 0])],
        );
        GlMesh::new(gl, &mesh)
    }

    fn position_graph(&self, ui: &mut Ui) {
        let position = Line::new(state_graph!(self.states, position))
            .color(Rgba::from_rgb(0.25, 0.75, 0.75))
            .name("Position");

        let velocity = Line::new(state_graph!(self.states, velocity))
            .color(Rgba::from_rgb(0.75, 0.75, 0.25))
            .name("Velocity");

        let acceleration = Line::new(state_graph!(self.states, acceleration))
            .color(Rgba::from_rgb(0.75, 0.25, 0.25))
            .name("Acceleration");

        ui.label("Kinematics");
        Plot::new("Kinematics graph")
            .data_aspect(self.bottom_data_aspect())
            .view_aspect(10.0)
            .auto_bounds_x()
            .auto_bounds_y()
            .legend(Self::bottom_legend())
            .show(ui, |plot_ui| {
                plot_ui.line(position);
                plot_ui.line(velocity);
                plot_ui.line(acceleration);
            });
    }

    fn forces_graph(&self, ui: &mut Ui) {
        let spring = Line::new(state_graph!(self.states, spring_force))
            .color(Rgba::from_rgb(0.0, 0.5, 0.75))
            .name("Spring");

        let damping = Line::new(state_graph!(self.states, damping_force))
            .color(Rgba::from_rgb(0.5, 0.75, 0.0))
            .name("Damping");

        let outer = Line::new(state_graph!(self.states, outer_force))
            .color(Rgba::from_rgb(0.75, 0.0, 0.5))
            .name("Outer");

        let total = Line::new(state_graph!(self.states, total_force))
            .color(Rgba::from_rgb(0.75, 0.75, 0.5))
            .name("Total");

        ui.label("Forces");
        Plot::new("Forces graph")
            .data_aspect(self.bottom_data_aspect())
            .view_aspect(10.0)
            .auto_bounds_x()
            .auto_bounds_y()
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(spring);
                plot_ui.line(damping);
                plot_ui.line(outer);
                plot_ui.line(total);
            });
    }

    fn state_space_graph(&self, ui: &mut Ui) {
        let sin: PlotPoints = self
            .states
            .iter()
            .map(|s| [s.position, s.velocity])
            .collect();

        let line = Line::new(sin)
            .color(Rgba::from_rgb(0.0, 0.5, 0.75))
            .name("State");

        ui.label("State space");
        Plot::new("State space graph")
            .data_aspect(1.0)
            .view_aspect(1.0)
            .auto_bounds_x()
            .auto_bounds_y()
            .legend(Self::bottom_legend())
            .show(ui, |plot_ui| plot_ui.line(line));
    }

    fn equilibrium_graph(&self, ui: &mut Ui) {
        let equilibrium = Line::new(state_graph!(self.states, equilibrium))
            .color(Rgba::from_rgb(0.25, 0.75, 0.75))
            .name("Equilibrium");

        ui.label("Equilibrium");
        Plot::new("Equilibrium graph")
            .data_aspect(self.bottom_data_aspect())
            .view_aspect(10.0)
            .auto_bounds_x()
            .auto_bounds_y()
            .legend(Self::bottom_legend())
            .show(ui, |plot_ui| plot_ui.line(equilibrium));
    }

    fn bottom_legend() -> Legend {
        Legend::default().position(Corner::RightTop)
    }

    fn bottom_data_aspect(&self) -> f32 {
        0.05 * self.states.last().map(|s| s.t - self.last_clear_t).unwrap_or(1.0) as f32
    }

    fn current_state(&self) -> Option<&SpringState> {
        self.states.last()
    }

    fn show_info(&self, ui: &mut Ui) {
        ui.label(format!("Steps so far: {}", self.states.len()));

        if let Some(state) = self.current_state() {
            for (name, val) in state.iter() {
                ui.label(format!("{name}: {val:.5}"));
            }
        }
    }

    fn current_outer_force(&self) -> &dyn ParametrizableFunction<F = spring::F> {
        self.outer_force_functions[self.selected_outer_force_idx].as_ref()
    }

    fn current_outer_force_mut(&mut self) -> &mut dyn ParametrizableFunction<F = spring::F> {
        self.outer_force_functions[self.selected_outer_force_idx].as_mut()
    }

    fn force_selection(&mut self, ui: &mut Ui) {
        let mut changed = self.current_outer_force_mut().manipulation_ui(ui);

        ComboBox::from_label("Outer force function")
            .selected_text(self.current_outer_force().name())
            .show_ui(ui, |ui| {
                for (i, f) in self.outer_force_functions.iter().enumerate() {
                    if ui
                        .selectable_value(&mut self.selected_outer_force_idx, i, f.name())
                        .clicked()
                    {
                        changed = true;
                    }
                }
            });

        if changed {
            *self.euler.ode_mut().outer_force_function_mut() =
                self.current_outer_force().produce_closure();
        }
    }

    fn clear_ui(&mut self, ui: &mut Ui) {
        if ui.button("Clear").clicked() {
            self.clear();
        }
    }

    fn clear(&mut self) {
        self.states.clear()
    }
}

impl Presenter for Spring {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        self.clear_ui(ui);
        self.show_info(ui);
        self.force_selection(ui);
        ui.vertical_centered(|ui| {
            self.state_space_graph(ui);
        });
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            self.position_graph(ui);
            self.forces_graph(ui);
            self.equilibrium_graph(ui);
        });
    }

    fn draw(&self, aspect_ratio: f32) {
        let Some(state) = self.states.last() else {
            return;
        };

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

        // Wall
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (na::geometry::Translation3::new(-0.5, 0.0, 0.0).to_homogeneous()
                * na::geometry::Scale3::new(0.1, 4.0, 1.0).to_homogeneous())
            .as_slice(),
        );
        self.rect_mesh.draw();

        // Spring
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (na::geometry::Translation3::new(-0.5, 0.0, 0.0).to_homogeneous()
                * na::geometry::Scale3::new(1.0 + 2.0 * state.position as f32, 0.1, 1.0)
                    .to_homogeneous()
                * na::geometry::Translation3::new(0.25, 0.0, 0.0).to_homogeneous())
            .as_slice(),
        );
        self.rect_mesh.draw();

        // Box
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (na::geometry::Translation3::new(state.position as f32, 0.0, 0.0).to_homogeneous()
                * na::geometry::Scale3::new(0.5, 0.5, 1.0).to_homogeneous())
            .as_slice(),
        );
        self.rect_mesh.draw();
    }

    fn update(&mut self) {
        self.states.reserve(self.steps_per_frame);
        for _ in 0..self.steps_per_frame {
            self.euler.step();
            self.states.push(self.euler.ode().state());
        }
    }

    fn name(&self) -> &'static str {
        "Spring"
    }
}
