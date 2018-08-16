use audio;
use gfx;
use gfx::gl;
use std::mem;
use std::ptr;
use visualizer::equalizer::EqualizerVisualizer;
use visualizer::power_circles::PowerCirclesVisualizer;
use visualizer::smiley::SmileyVisualizer;

pub struct Visualizer {
    framebuffer_id: u32,
    texture_id: u32,

    equalizer_visualizer: EqualizerVisualizer,
    power_circles_visualizer: PowerCirclesVisualizer,
    smiley_visualizer: SmileyVisualizer,

    selected_visualizer: String,
}

impl Visualizer {
    pub fn new(selected_visualizer: String) -> Visualizer {
        Visualizer {
            framebuffer_id: 0,
            texture_id: 0,

            equalizer_visualizer: EqualizerVisualizer::new(),
            power_circles_visualizer: PowerCirclesVisualizer::new(),
            smiley_visualizer: SmileyVisualizer::new(),

            selected_visualizer,
        }
    }

    pub fn setup(&mut self, gl: &gfx::gl::Gl) {
        unsafe {
            let mut texture = mem::uninitialized();
            gl_try!(gl; gl.GenTextures(1, &mut texture));
            gl_try!(gl; gl.BindTexture(gl::TEXTURE_2D, texture));
            gl_try!(gl; gl.ActiveTexture(gl::TEXTURE0));

            gl_try!(gl; gl.TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32
            ));
            gl_try!(gl; gl.TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32
            ));

            gl_try!(gl; gl.TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGB as i32,
                1024, 1024, 0, gl::RGB, gl::UNSIGNED_BYTE,
                ptr::null(),
            ));

            self.texture_id = texture;

            let mut framebuffer = mem::uninitialized();
            gl_try!(gl; gl.GenFramebuffers(1, &mut framebuffer));
            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer));
            gl_try!(gl; gl.FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture, 0,
            ));

            self.framebuffer_id = framebuffer;

            self.equalizer_visualizer.setup(gl, self.framebuffer_id);
            self.power_circles_visualizer.setup(gl, self.framebuffer_id);
            self.smiley_visualizer.setup(gl, self.framebuffer_id);
        }
    }

    pub fn update(&mut self, audio_frame: audio::AudioFrame) {
        self.equalizer_visualizer.update(audio_frame.clone());
        self.power_circles_visualizer.update(audio_frame.clone());
        self.smiley_visualizer.update(audio_frame.clone());
    }

    pub fn render_to_texture(&self, gl: &gfx::gl::Gl) -> u32 {
        match self.selected_visualizer.as_ref() {
            "equalizer" => self.equalizer_visualizer.render_to_texture(gl),
            "power_circles" => self.power_circles_visualizer.render_to_texture(gl),
            "smiley" => self.smiley_visualizer.render_to_texture(gl),

            _ => self.power_circles_visualizer.render_to_texture(gl),
        }

        self.texture_id
    }
}
