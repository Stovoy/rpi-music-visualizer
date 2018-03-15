mod gfx;
mod rpio;

extern crate gl;
extern crate glutin;
extern crate sysfs_gpio;

use glutin::GlContext;

fn main() {
    println!("=== Starting Music Visualizer ===");
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Music Visualizer")
        .with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    let _ = unsafe { gl_window.make_current() };

    let gl = gfx::load(&gl_window);

    events_loop.run_forever(|event| {
        println!("{:?}", event);
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => return glutin::ControlFlow::Break,
                glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                _ => (),
            },
            _ => ()
        }

        gl.draw_frame([0.0, 1.0, 0.0, 1.0]);
        let _ = gl_window.swap_buffers();
        glutin::ControlFlow::Continue
    });
}
