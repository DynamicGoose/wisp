use glam::{Quat, Vec3};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};
use wisp::{
    camera::{Camera, Viewport},
    instance::Instance,
    RenderState,
};

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let window = Window::new(&event_loop).unwrap();

    let mut state = pollster::block_on(RenderState::new(&window));
    let camera = Camera {
        // position the camera 1 unit up and 2 units back
        // +z is out of the screen
        eye: (0.0, 20.0, 0.01).into(),
        // have it look at the origin
        target: (0.0, 0.0, 0.0).into(),
        // which way is "up"
        up: Vec3::Y,
        fovy: 96.0,
        znear: 0.1,
        zfar: 100.0,
        viewport: Some(Viewport {
            x: 0.0,
            y: 0.0,
            w: 128.0,
            h: 128.0,
        }),
    };
    state.add_camera(camera);

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

    // Adding and deleting Models
    let model = pollster::block_on(state.load_model_instanced("cube.obj", vec![]));
    state.remove_model(model);
    let model = pollster::block_on(state.load_model_instanced("cube.obj", instances));

    // Pushing an Instance
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
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                elwt.exit();
            }
            Event::AboutToWait => {
                counter += 1;
                if counter > 1000 {
                    elwt.exit();
                }

                // Updating instances
                let instance = state.get_instance(model, 2);
                let instance_override = Instance {
                    position: Vec3::new(
                        instance.position.x + 0.01,
                        instance.position.y + 0.01,
                        instance.position.z,
                    ),
                    rotation: instance.rotation,
                };

                state.override_instance(0, 2, instance_override);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                state.render().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                state.resize(new_size);
            }
            _ => (),
        })
        .unwrap();

    println!("{}", current_time.elapsed().unwrap().as_millis());
}
