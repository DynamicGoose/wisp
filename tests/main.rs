use glam::{Quat, Vec3};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};
use wisp::{instance::Instance, RenderState};

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let window = Window::new(&event_loop).unwrap();

    let mut state = pollster::block_on(RenderState::new(&window));

    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const SPACE_BETWEEN: f32 = 3.0;
    let instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = glam::Vec3 { x, y: 0.0, z };

                let rotation = if position == glam::Vec3::ZERO {
                    glam::Quat::from_axis_angle(glam::Vec3::Z, 0.0)
                } else {
                    glam::Quat::from_axis_angle(position.normalize(), 45.0)
                };

                Instance { position, rotation }
            })
        })
        .collect::<Vec<_>>();
    let model = pollster::block_on(state.load_model_instanced("cube.obj", instances));

    state.push_instance(
        model,
        Instance {
            position: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::from_axis_angle(Vec3::Z, 0.0),
        },
    );

    let mut counter = 0;

    let current_time = std::time::SystemTime::now();
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("The close button was pressed; stopping");
                    elwt.exit();
                }
                Event::AboutToWait => {
                    // Application update code.

                    // Queue a RedrawRequested event.
                    //
                    // You only need to call this if you've determined that you need to redraw in
                    // applications which do not always need to. Applications that redraw continuously
                    // can render here instead.
                    counter += 1;
                    if counter > 1000 {
                        elwt.exit();
                    }
                    state.override_instance(
                        0,
                        2,
                        Instance {
                            position: Vec3::new(counter as f32 * 0.01, counter as f32 * 0.01, 0.0),
                            rotation: Quat::from_array([0.0, 0.0, 0.0, 0.0]),
                        },
                    );
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    state.render().unwrap();
                    // Redraw the application.
                    //
                    // It's preferable for applications that do not render continuously to render in
                    // this event rather than in AboutToWait, since rendering in here allows
                    // the program to gracefully handle redraws requested by the OS.
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    state.resize(new_size);
                }
                _ => (),
            }
        })
        .unwrap();

    println!("{}", current_time.elapsed().unwrap().as_millis());
}
