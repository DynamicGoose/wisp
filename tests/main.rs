use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};
use wisp::RenderState;

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let window = Window::new(&event_loop).unwrap();

    let mut state = pollster::block_on(RenderState::new(&window));

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::AboutToWait => {
                    state.window.request_redraw();
                }
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            println!("The close button was pressed; stopping");
                            elwt.exit();
                        }
                        WindowEvent::RedrawRequested => {
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.surface_size),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        WindowEvent::Resized(physicsl_size) => {
                            state.resize(physicsl_size);
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            state.resize(PhysicalSize {
                                width: state.window.inner_size().width
                                    * scale_factor.round() as u32,
                                height: state.window.inner_size().height
                                    * scale_factor.round() as u32,
                            })
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        })
        .unwrap();
}
