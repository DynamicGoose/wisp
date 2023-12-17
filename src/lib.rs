use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Wisp Window")
        .build(&event_loop)
        .unwrap();

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
                    // You only need to call this if you've determined that you need to redraw, in
                    // applications which do not always need to. Applications that redraw continuously
                    // can just render here instead.
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // Redraw the application.
                    //
                    // It's preferable for applications that do not render continuously to render in
                    // this event rather than in AboutToWait, since rendering in here allows
                    // the program to gracefully handle redraws requested by the OS.
                }
                _ => (),
            }
        })
        .unwrap();
}
