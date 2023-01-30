pub mod config;
pub mod rendering;
pub mod simulation;
pub mod timing;

use winit::{
    event::Event::*,
    event_loop::{ControlFlow, EventLoop},
};

use crate::{config::build_config, rendering::RenderDriver, simulation::Simulation};

fn main() {
    let mut simulation = Simulation::new(build_config());

    let event_loop = EventLoop::new();
    let mut renderer = RenderDriver::new(&mut simulation, &event_loop);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Forward event to renderers
        renderer.handle_event(&mut simulation, &event);

        match event {
            // Rendering
            RedrawRequested(..) => renderer.render(&mut simulation),
            MainEventsCleared => {
                if simulation.should_step() {
                    simulation.step();
                }
                if renderer.should_redraw(simulation.get_config()) {
                    renderer.request_render()
                }
            }
            // Handle changes to wndow
            WindowEvent { event, .. } => renderer.handle_window_event(&event, control_flow),
            _ => {}
        }
    });
}
