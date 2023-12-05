use egui::containers::ComboBox;
use egui_winit::winit::{self, platform::run_return::EventLoopExtRunReturn};
use phyesthon::{
    controls::mouse::MouseState,
    presenters::{
        jelly::JellyBuilder, kinematic_chain::KinematicChainBuilder,
        quaternions::QuaternionsBuilder, spinning_top::SpinningTopBuilder, spring::SpringBuilder,
        Presenter, PresenterBuilder,
    },
    window::Window,
};
use std::time::Instant;

fn main() {
    let mut mouse = MouseState::new();
    let mut event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let window = unsafe { Window::new(&event_loop) };

    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, window.clone_gl(), None);
    egui_extras::install_image_loaders(&mut egui_glow.egui_ctx);

    let mut builders: Vec<Box<dyn PresenterBuilder>> = vec![
        Box::new(KinematicChainBuilder::new()),
        Box::new(JellyBuilder::new()),
        Box::new(QuaternionsBuilder::new()),
        Box::new(SpinningTopBuilder::new()),
        Box::new(SpringBuilder::new()),
    ];


    let mut presenters: Vec<Box<dyn Presenter>> = builders
        .iter()
        .map(|builder| builder.build(window.clone_gl()))
        .collect();

    let mut current_presenter = 0;
    let mut auto_reset = true;

    let mut pause = true;
    let mut last_draw = None;

    event_loop.run_return(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(_) => {
            render(
                &mut egui_glow,
                &mut current_presenter,
                &mut presenters,
                &mut builders,
                &window,
                &mut pause,
                &mut mouse,
                &mut last_draw,
                &mut auto_reset,
            );
        }
        winit::event::Event::WindowEvent { event, .. } => {
            use winit::event::WindowEvent;
            if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }

            if let winit::event::WindowEvent::Resized(physical_size) = &event {
                window.resize(*physical_size);
            } else if let winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } =
                &event
            {
                window.resize(**new_inner_size);
            }

            let event_response = egui_glow.on_event(&event);

            if event_response.repaint {
                window.window().request_redraw();
            }

            if !event_response.consumed {
                mouse.handle_window_event(&event);
            }
        }
        winit::event::Event::LoopDestroyed => {
            egui_glow.destroy();
        }
        winit::event::Event::NewEvents(winit::event::StartCause::ResumeTimeReached { .. }) => {
            window.window().request_redraw();
        }
        _ => window.window().request_redraw(),
    });
}

fn render(
    egui_glow: &mut egui_glow::EguiGlow,
    current_presenter: &mut usize,
    presenters: &mut [Box<dyn Presenter>],
    builders: &mut [Box<dyn PresenterBuilder>],
    window: &Window,
    paused: &mut bool,
    mouse: &mut MouseState,
    last_draw: &mut Option<Instant>,
    auto_reset: &mut bool,
) {
    let now = Instant::now();
    let delta = last_draw.map(|last| now - last);

    if !*paused {
        if let Some(delta) = delta {
            presenters[*current_presenter].update(delta);
        }
    }

    *last_draw = Some(now);

    presenters[*current_presenter].update_mouse(*mouse);
    mouse.update();

    let repaint_after = egui_glow.run(window.window(), |egui_ctx| {
        draw_ui(
            current_presenter,
            presenters,
            builders,
            window,
            paused,
            egui_ctx,
            auto_reset,
        );
    });

    if repaint_after.is_zero() {
        window.window().request_redraw();
        winit::event_loop::ControlFlow::Poll
    } else if let Some(repaint_after_instant) = std::time::Instant::now().checked_add(repaint_after)
    {
        winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
    } else {
        winit::event_loop::ControlFlow::Wait
    };

    window.clear();

    presenters[*current_presenter].draw(window.size());

    egui_glow.paint(window.window());

    // draw things on top of egui here

    window.swap_buffers().unwrap();
    window.window().set_visible(true);
}

fn draw_ui(
    current_presenter: &mut usize,
    presenters: &mut [Box<dyn Presenter>],
    builders: &mut [Box<dyn PresenterBuilder>],
    window: &Window,
    paused: &mut bool,
    egui_ctx: &egui::Context,
    auto_reset: &mut bool,
) {
    egui::SidePanel::left("Side panel")
        .min_width(100.0)
        .max_width(500.0)
        .default_width(400.0)
        .show(egui_ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ComboBox::from_label("Presenter selection")
                    .selected_text(presenters[*current_presenter].name())
                    .show_ui(ui, |ui| {
                        for (i, f) in presenters.iter().enumerate() {
                            if ui
                                .selectable_value(current_presenter, i, f.name())
                                .clicked()
                            {
                                *paused = true;
                            }
                        }
                    });

                ui.heading(presenters[*current_presenter].name());
                let text = if *paused { "Play" } else { "Pause" };
                if ui.button(text).clicked() {
                    *paused = !*paused;
                }

                ui.separator();

                let changed = builders[*current_presenter].build_ui(ui).changed();
                ui.checkbox(auto_reset, "Autoreset");
                if ui.button("Reset").clicked() || changed && *auto_reset {
                    presenters[*current_presenter] =
                        builders[*current_presenter].build(window.clone_gl());
                }

                ui.separator();

                presenters[*current_presenter].show_side_ui(ui);
            })
        });

    egui::TopBottomPanel::bottom("Bottom panel")
        .max_height(400.0)
        .min_height(100.0)
        .default_height(300.0)
        .show(egui_ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                presenters[*current_presenter].show_bottom_ui(ui);
            })
        });
}
