use super::{Presenter, PresenterBuilder};
use crate::{
    controls::mouse::MouseState,
    render::{
        gl_drawable::GlDrawable,
        gl_mesh::{GlLineStrip, GlLines, GlTriangleMesh},
        gl_program::GlProgram,
        mesh::{Mesh, Triangle},
        models,
    },
};
use egui::{Rgba, Ui};
use egui_plot::{Line, Plot};
use itertools::Itertools;
use nalgebra as na;
use rand_distr::Distribution;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct Hodograph {
    gl_program: GlProgram,
    rect_mesh: GlTriangleMesh,
    circle_mesh: GlLineStrip,
    arm_mesh: GlLines,
    radius_mesh: GlLines,

    stddev: f64,
    dist: rand_distr::Normal<f64>,
    rng: rand::rngs::ThreadRng,

    angular_speed: f64,
    arm_length: f64,
    wheel_radius: f64,
    delta: f64,
    work_left: f64,
    error: f64,

    x: VecDeque<f64>,
    xp: VecDeque<f64>,
    xpp: VecDeque<f64>,
    time: VecDeque<f64>,

    xaspect: RefCell<f32>,
    xpaspect: RefCell<f32>,
    xppaspect: RefCell<f32>,
    xxxxaspect: RefCell<f32>,

    angle: f64,

    simulation_speed: f64,
}

impl Hodograph {
    const MAX_HISTORY: usize = 10000;

    pub fn new(gl: Arc<glow::Context>) -> Self {
        let mut me = Hodograph {
            rect_mesh: Self::create_rect_mesh(Arc::clone(&gl)),
            circle_mesh: GlLineStrip::new(Arc::clone(&gl), &models::circle(100)),
            radius_mesh: GlLines::new(
                Arc::clone(&gl),
                &[na::Point3::origin(), na::Point3::origin()],
            ),
            arm_mesh: GlLines::new(
                Arc::clone(&gl),
                &[na::Point3::origin(), na::Point3::origin()],
            ),
            gl_program: GlProgram::vertex_fragment(Arc::clone(&gl), "2d_vert", "pass_frag"),

            stddev: 0.000001,
            dist: rand_distr::Normal::new(0.0, 0.000001).unwrap(),
            rng: rand::thread_rng(),

            angular_speed: 1.0,
            arm_length: 0.8,
            wheel_radius: 0.25,
            delta: 0.01,
            work_left: 0.0,
            error: 0.0,

            time: VecDeque::from([0.0]),
            x: VecDeque::new(), // Assigned later
            xp: VecDeque::new(),
            xpp: VecDeque::new(),
            xaspect: RefCell::new(1.0),
            xpaspect: RefCell::new(1.0),
            xppaspect: RefCell::new(1.0),
            xxxxaspect: RefCell::new(1.0),

            angle: 0.0,

            simulation_speed: 1.0,
        };

        me.radius_mesh.update_points(&me.radius_points());
        me.arm_mesh.update_points(&me.arm_points());
        me.x.push_back(me.slide());
        me
    }

    fn error(&mut self) -> f64 {
        self.dist.sample(&mut self.rng)
    }

    fn slide(&self) -> f64 {
        (self.angle.sin() * self.wheel_radius / (self.arm_length + self.error))
            .asin()
            .abs()
            .cos()
            * (self.arm_length + self.error)
            + self.angle.cos() * self.wheel_radius
    }

    fn create_rect_mesh(gl: Arc<glow::Context>) -> GlTriangleMesh {
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
        GlTriangleMesh::new(gl, &mesh)
    }

    fn radius_points(&self) -> [na::Point3<f32>; 2] {
        [
            na::point![0.0, 0.0, 0.0],
            na::point![
                (self.wheel_radius * self.angle.cos()) as f32,
                (self.wheel_radius * self.angle.sin()) as f32,
                0.0
            ],
        ]
    }

    fn arm_points(&self) -> [na::Point3<f32>; 2] {
        [
            na::point![
                (self.wheel_radius * self.angle.cos()) as f32,
                (self.wheel_radius * self.angle.sin()) as f32,
                0.0
            ],
            na::point![self.slide() as f32, 0.0, 0.0],
        ]
    }

    fn plot(
        &self,
        ui: &mut Ui,
        color: Rgba,
        name: &str,
        argument: &VecDeque<f64>,
        variable: &VecDeque<f64>,
        shift: usize,
        aspect: &RefCell<f32>,
    ) {
        let line = Line::new(
            argument
                .iter()
                .skip(shift)
                .zip(variable.iter())
                .map(|(&t, &x)| [t, x])
                .collect_vec(),
        )
        .color(color)
        .name(name);

        ui.vertical(|ui| {
            ui.label(name);
            ui.add(egui::Slider::new(&mut *aspect.borrow_mut(), 0.01..=100.0).logarithmic(true));
            Plot::new(name)
                .data_aspect(*aspect.borrow())
                .view_aspect(1.0)
                .width(350.0)
                .height(350.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
        });
    }
}

impl Presenter for Hodograph {
    fn show_side_ui(&mut self, ui: &mut Ui) {
        ui.label("ε0");
        if ui
            .add(
                egui::DragValue::new(&mut self.stddev)
                    .clamp_range(0.0..=0.0001)
                    .speed(0.00000001),
            )
            .changed()
        {
            self.dist = rand_distr::Normal::new(0.0, self.stddev).unwrap();
        }

        ui.label("ω");
        ui.add(
            egui::DragValue::new(&mut self.angular_speed)
                .clamp_range(-10.0..=10.0)
                .speed(0.1),
        );

        ui.label("L");
        ui.add(
            egui::DragValue::new(&mut self.arm_length)
                .clamp_range(self.wheel_radius..=1.0)
                .speed(0.01),
        );

        ui.label("R");
        ui.add(
            egui::DragValue::new(&mut self.wheel_radius)
                .clamp_range(0.05..=1.0)
                .speed(0.01),
        );

        ui.label("Δ");
        ui.add(
            egui::DragValue::new(&mut self.delta)
                .clamp_range(0.001..=0.1)
                .speed(0.001),
        );

        ui.label("Simulation Speed");
        ui.add(
            egui::DragValue::new(&mut self.simulation_speed)
                .clamp_range(0.0..=10.0)
                .speed(0.1),
        );

        self.plot(
            ui,
            Rgba::from_rgb(0.8, 0.8, 0.8),
            "State",
            &self.x,
            &self.xp,
            0,
            &self.xxxxaspect,
        );
    }

    fn show_bottom_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.plot(
                ui,
                Rgba::from_rgb(0.1, 0.1, 1.0),
                "Position",
                &self.time,
                &self.x,
                0,
                &self.xaspect,
            );
            self.plot(
                ui,
                Rgba::from_rgb(0.1, 1.0, 0.1),
                "Velocity",
                &self.time,
                &self.xp,
                1,
                &self.xpaspect,
            );
            self.plot(
                ui,
                Rgba::from_rgb(1.0, 0.1, 0.1),
                "Acceleration",
                &self.time,
                &self.xpp,
                1,
                &self.xppaspect,
            );
        });
    }

    fn draw(&self, size: Option<egui_winit::winit::dpi::PhysicalSize<u32>>) {
        let Some(size) = size else { return };
        let aspect_ratio = size.width as f32 / size.height as f32;

        self.gl_program.enable();

        self.gl_program.uniform_matrix_4_f32_slice(
            "view_transform",
            na::matrix![
                1.0 / aspect_ratio, 0.0, 0.0, -0.3;
                0.0, 1.0, 0.0, 0.4;
                0.0, 0.0, 1.0, 0.0;
                0.0, 0.0, 0.0, 1.0;
            ]
            .as_slice(),
        );

        // Piston
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            (na::geometry::Translation3::new(self.slide() as f32, 0.0, 0.0).to_homogeneous()
                * na::geometry::Scale3::new(0.125, 0.125, 1.0).to_homogeneous())
            .as_slice(),
        );

        self.rect_mesh.draw();

        // Wheel
        self.gl_program.uniform_matrix_4_f32_slice(
            "model_transform",
            na::geometry::Scale3::new(self.wheel_radius as f32, self.wheel_radius as f32, 1.0)
                .to_homogeneous()
                .as_slice(),
        );

        self.circle_mesh.draw();

        // Lines
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", na::Matrix4::identity().as_slice());
        self.radius_mesh.draw();
        self.arm_mesh.draw();
    }

    fn update(&mut self, delta: std::time::Duration) {
        self.work_left += delta.as_secs_f64() * self.simulation_speed;
        while self.work_left >= self.delta {
            self.error = self.error();
            self.angle += self.angular_speed * self.delta;

            self.time.push_back(self.delta + self.time.back().unwrap());
            self.x.push_back(self.slide());

            if self.x.len() >= 3 {
                self.xp.push_back(
                    (self.x[self.x.len() - 1] - self.x[self.x.len() - 3]) / 2.0 / self.delta,
                );
            }

            if self.xp.len() >= 3 {
                self.xpp.push_back(
                    (self.xp[self.xp.len() - 1] - self.xp[self.xp.len() - 3]) / 2.0 / self.delta,
                );
            }

            if self.xpp.len() > Self::MAX_HISTORY {
                let to_remove = self.xpp.len() - Self::MAX_HISTORY;
                self.time.drain(0..=to_remove);
                self.x.drain(0..=to_remove);
                self.xp.drain(0..=to_remove);
                self.xpp.drain(0..=to_remove);
            }

            self.work_left -= self.delta;
        }

        self.radius_mesh.update_points(&self.radius_points());
        self.arm_mesh.update_points(&self.arm_points());
    }

    fn name(&self) -> &'static str {
        "Hodograph"
    }

    fn update_mouse(&mut self, _state: MouseState) {}
}

pub struct HodographBuilder {}

impl HodographBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenterBuilder for HodographBuilder {
    fn build_ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("Hodograph")
    }

    fn build(&self, gl: Arc<glow::Context>) -> Box<dyn Presenter> {
        Box::new(Hodograph::new(gl))
    }
}

impl Default for HodographBuilder {
    fn default() -> Self {
        Self {}
    }
}
