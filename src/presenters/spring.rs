use super::Presenter;
use egui::{Rgba, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use itertools::Itertools;
use std::sync::Arc;

type Float = f32;

macro_rules! state_graph {
    ($states:expr, $field:ident) => {
        $states
            .iter()
            .enumerate()
            .map(|(t, s)| [t as f64, s.$field as f64])
            .collect_vec()
    };
}

pub struct SpringState {
    position: Float,
    velocity: Float,
    acceleration: Float,

    elastic_force: Float,
    damping_force: Float,
    outer_force: Float,

    spring_tip: Float,
}

pub struct Spring {
    gl: Arc<glow::Context>,
    states: Vec<SpringState>,
}

impl Spring {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Spring {
            gl,
            states: Vec::new(),
        }
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

        let legend = Legend::default();

        ui.label("Kinematics");
        Plot::new("Kinematics graph")
            .data_aspect(10.0)
            .view_aspect(10.0)
            .auto_bounds_x()
            .auto_bounds_y()
            .legend(legend)
            .show(ui, |plot_ui| {
                plot_ui.line(position);
                plot_ui.line(velocity);
                plot_ui.line(acceleration);
            });
    }

    fn forces_graph(&self, ui: &mut Ui) {
        let elastic = Line::new(state_graph!(self.states, elastic_force))
            .color(Rgba::from_rgb(0.0, 0.5, 0.75))
            .name("Elastic");

        let damping = Line::new(state_graph!(self.states, damping_force))
            .color(Rgba::from_rgb(0.5, 0.75, 0.0))
            .name("Damping");

        let outer = Line::new(state_graph!(self.states, outer_force))
            .color(Rgba::from_rgb(0.75, 0.0, 0.5))
            .name("Outer");

        let legend = Legend::default();

        ui.label("Forces");
        Plot::new("Forces graph")
            .data_aspect(10.0)
            .view_aspect(10.0)
            .auto_bounds_x()
            .auto_bounds_y()
            .legend(legend)
            .show(ui, |plot_ui| {
                plot_ui.line(elastic);
                plot_ui.line(damping);
                plot_ui.line(outer);
            });
    }

    fn state_space_graph(&self, ui: &mut Ui) {
        let sin: PlotPoints = (0..1000)
            .map(|i| {
                let x = i as f64 * 0.01;
                [x.cos(), x.sin()]
            })
            .collect();

        let line = Line::new(sin)
            .color(Rgba::from_rgb(0.0, 0.5, 0.75))
            .name("State");

        let legend = Legend::default();

        ui.label("State space");
        Plot::new("State space graph")
            .data_aspect(1.0)
            .view_aspect(1.0)
            .auto_bounds_x()
            .auto_bounds_y()
            .legend(legend)
            .show(ui, |plot_ui| plot_ui.line(line));
    }
}

impl Presenter for Spring {
    fn show_side_ui(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            self.state_space_graph(ui);
        });
    }

    fn show_bottom_ui(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            self.position_graph(ui);
            self.forces_graph(ui);
            // TODO: w(t)
        });
    }

    fn draw(&self) {}

    fn update(&mut self) {}

    fn name(&self) -> &'static str {
        "Spring"
    }
}
