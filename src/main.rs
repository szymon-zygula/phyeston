use egui_winit::winit::{self, platform::run_return::EventLoopExtRunReturn};
use phyesthon::{
    presenters::{spring::Spring, Presenter},
    window::Window,
};

fn main() {
    let mut event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let window = unsafe { Window::new(&event_loop) };

    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, window.clone_gl(), None);

    let spring = Spring::new(window.clone_gl(), 0.1, 1);
    let mut presenters: [Box<dyn Presenter>; 1] = [Box::new(spring)];

    let mut pause = false;
    let current_presenter = presenters[0].as_mut();

    event_loop.run_return(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(_) => {
            render(&mut egui_glow, current_presenter, &window, &mut pause);
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
    current_presenter: &mut dyn Presenter,
    window: &Window,
    pause: &mut bool,
) {
    if !*pause {
        current_presenter.update();
    }

    let repaint_after = egui_glow.run(window.window(), |egui_ctx| {
        egui::SidePanel::left("Side panel")
            .min_width(100.0)
            .max_width(500.0)
            .default_width(400.0)
            .show(egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(current_presenter.name());
                    if ui.button("Pause").clicked() {
                        *pause = !*pause;
                    }

                    ui.separator();

                    current_presenter.show_side_ui(ui);
                })
            });

        egui::TopBottomPanel::bottom("Bottom panel")
            .max_height(400.0)
            .min_height(100.0)
            .default_height(300.0)
            .show(egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    current_presenter.show_bottom_ui(ui);
                })
            });
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

    let aspect_ratio = window
        .size()
        .map(|p| p.width as f32 / p.height as f32)
        .unwrap_or(1.0);
    current_presenter.draw(aspect_ratio);

    egui_glow.paint(window.window());

    // draw things on top of egui here

    window.swap_buffers().unwrap();
    window.window().set_visible(true);
}
