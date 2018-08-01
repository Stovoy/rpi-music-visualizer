use std::os::raw::c_void;
use std::sync::mpsc;

use glutin;
use glutin::GlContext;

use audio;
use visualizer;
use screen;

pub mod gl {
    pub use self::Gles2 as Gl;
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

macro_rules! gl_try {
    ($gl:expr; $call:expr) => {{
        let result = $call;
        // TODO: Compile away with a compile-flag.
        let gl_err = $gl.GetError();
        if gl_err != gl::NO_ERROR {
            panic!("gl error: {} (0x{:X})", gl_err, gl_err);
        }

        result
    }}
}

pub fn run(visualizer: visualizer::Visualizer,
           screen: Box<screen::Screen>,
           audio_rx: mpsc::Receiver<audio::AudioFrame>) {
    if screen.uses_window() {
        render_with_window(visualizer, screen, audio_rx);
    } else {
        render_without_window(visualizer, screen, audio_rx);
    }
}

fn render_with_window(visualizer: visualizer::Visualizer,
                      screen: Box<screen::Screen>,
                      audio_rx: mpsc::Receiver<audio::AudioFrame>) {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Music Visualizer")
        .with_dimensions(1024, 1024);
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    let mut pipeline = GfxPipeline::new(load_gl_window_as_context(&gl_window), visualizer, screen);

    let mut running = true;
    while running {
        let audio_frame = match audio_rx.recv() {
            Ok(x) => x,
            Err(error) => continue,
        };

        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                _ => (),
            },
            _ => (),
        });

        pipeline.update(audio_frame);
        gl_window.swap_buffers().unwrap();
    }
}

fn render_without_window(visualizer: visualizer::Visualizer,
                         screen: Box<screen::Screen>,
                         audio_rx: mpsc::Receiver<audio::AudioFrame>) {
    let window = glutin::WindowBuilder::new()
        .with_title("Music Visualizer")
        .with_visibility(false);
    let context = glutin::ContextBuilder::new();
    let gl_window = glutin::GlWindow::new(window, context, &glutin::EventsLoop::new()).unwrap();

    let mut pipeline = GfxPipeline::new(load_gl_window_as_context(&gl_window),
                                        visualizer, screen);

    loop {
        let audio_frame = match audio_rx.recv() {
            Ok(x) => x,
            Err(error) => continue,
        };

        pipeline.update(audio_frame);
    }
}

pub fn load_gl_window_as_context(gl_window: &glutin::GlWindow) -> gl::Gl {
    unsafe { gl_window.make_current() }.unwrap();
    let gl = gl::Gl::load_with(|ptr| gl_window.get_proc_address(ptr) as *const _);

    unsafe {
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl.Disable(gl::DEPTH_TEST);
    }

    gl
}


pub struct GfxPipeline {
    gl: gl::Gl,
    visualizer: visualizer::Visualizer,
    screen: Box<screen::Screen>,
}

impl GfxPipeline {
    pub fn new(
        gl: gl::Gl,
        mut visualizer: visualizer::Visualizer,
        mut screen: Box<screen::Screen>,
    ) -> GfxPipeline {
        visualizer.setup(&gl);
        screen.setup(&gl);

        let pipeline = GfxPipeline {
            gl,
            visualizer,
            screen,
        };

        pipeline
    }

    pub fn update(&mut self, audio_frame: audio::AudioFrame) {
        self.visualizer.update(audio_frame);

        unsafe {
            let gl = &self.gl;
            gl_try!(gl; gl.ClearColor(0.0, 0.0, 0.0, 1.0));
            gl_try!(gl; gl.Clear(gl::COLOR_BUFFER_BIT));

            gl_try!(gl; gl.Viewport(0, 0, 1024, 1024));
            let texture = self.visualizer.render_to_texture(gl);
            // TODO: Scale based on display size (pixel density / resolution mismatch).
            gl_try!(gl; gl.Viewport(0, 0, 1024, 1024));
            self.screen.render_from_texture(gl, texture);
        }
    }

    #[allow(dead_code)]
    fn read_pixels(&self, width: usize, height: usize) -> Vec<u8> {
        let mut pixels = vec![0 as u8; 3 * width * height];

        unsafe {
            self.gl.ReadPixels(
                0, 0,
                width as i32, height as i32,
                gl::RGB, gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr() as *mut c_void);
        }

        pixels
    }
}
