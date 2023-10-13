use egui_winit::winit;
use phyesthon::window::Window;
use std::sync::Arc;

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let window = unsafe { Window::new(&event_loop) };

    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, Arc::clone(window.gl()), None);

    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::RedrawRequested(_) => {
                let repaint_after = egui_glow.run(window.window(), |egui_ctx| {
                    egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                        ui.heading("Hello World!");
                        if ui.button("Nice").clicked() {
                            println!("Nice!");
                        }
                    });
                });

                if repaint_after.is_zero() {
                    window.window().request_redraw();
                    winit::event_loop::ControlFlow::Poll
                } else if let Some(repaint_after_instant) =
                    std::time::Instant::now().checked_add(repaint_after)
                {
                    winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
                } else {
                    winit::event_loop::ControlFlow::Wait
                };

                window.clear();

                // draw things behind egui here

                egui_glow.paint(window.window());

                // draw things on top of egui here

                window.swap_buffers().unwrap();
                window.window().set_visible(true);
            }
            winit::event::Event::WindowEvent { event, .. } => {
                use winit::event::WindowEvent;
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }

                if let winit::event::WindowEvent::Resized(physical_size) = &event {
                    window.resize(*physical_size);
                } else if let winit::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size, ..
                } = &event
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
            winit::event::Event::NewEvents(winit::event::StartCause::ResumeTimeReached {
                ..
            }) => {
                window.window().request_redraw();
            }

            _ => (),
        }
    });
}
