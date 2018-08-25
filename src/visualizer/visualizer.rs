use audio;
use gfx;
use gfx::gl;
use std::mem;
use std::ptr;
use visualizer::equalizer::EqualizerVisualizer;
use visualizer::power_circles::PowerCirclesVisualizer;
use visualizer::smiley::SmileyVisualizer;
use visualizer::symmetry::BiSymmetryVisualizer;
use visualizer::symmetry::TriSymmetryVisualizer;
use visualizer::symmetry::QuadSymmetryVisualizer;
use visualizer::symmetry::PentaSymmetryVisualizer;

pub trait SubVisualizer {
    fn new() -> Self where Self: Sized;
    fn post_setup(&mut self, program_id: u32, framebuffer_id: u32);
    fn update(&mut self, audio_frame: audio::AudioFrame);
    fn render_to_texture(&self, gl: &gfx::gl::Gl);
    fn vs_src(&self) -> &[u8];
    fn fs_src(&self) -> &[u8];
}

pub struct Visualizer {
    texture_id: u32,

    equalizer_visualizer: EqualizerVisualizer,
    power_circles_visualizer: PowerCirclesVisualizer,
    smiley_visualizer: SmileyVisualizer,
    bisymmetry_visualizer: BiSymmetryVisualizer,
    trisymmetry_visualizer: TriSymmetryVisualizer,
    quadsymmetry_visualizer: QuadSymmetryVisualizer,
    pentasymmetry_visualizer: PentaSymmetryVisualizer,

    selected_visualizer: String,
}

impl Visualizer {
    pub fn new(selected_visualizer: String) -> Visualizer {
        Visualizer {
            texture_id: 0,

            equalizer_visualizer: EqualizerVisualizer::new(),
            power_circles_visualizer: PowerCirclesVisualizer::new(),
            smiley_visualizer: SmileyVisualizer::new(),
            bisymmetry_visualizer: BiSymmetryVisualizer::new(),
            trisymmetry_visualizer: TriSymmetryVisualizer::new(),
            quadsymmetry_visualizer: QuadSymmetryVisualizer::new(),
            pentasymmetry_visualizer: PentaSymmetryVisualizer::new(),

            selected_visualizer,
        }
    }

    pub fn setup(&mut self, gl: &gfx::gl::Gl, size: i32) {
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
                size, size, 0, gl::RGB, gl::UNSIGNED_BYTE,
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

            let mut all_visualizers: [&mut SubVisualizer; 7] = [
                &mut self.equalizer_visualizer,
                &mut self.power_circles_visualizer,
                &mut self.smiley_visualizer,
                &mut self.bisymmetry_visualizer,
                &mut self.trisymmetry_visualizer,
                &mut self.quadsymmetry_visualizer,
                &mut self.pentasymmetry_visualizer];
            for visualizer in all_visualizers.iter_mut() {
                let (program_id, framebuffer_id) = visualizer.setup(gl, framebuffer);
                visualizer.post_setup(program_id, framebuffer_id);
            }
        }
    }

    pub fn update(&mut self, audio_frame: audio::AudioFrame) {
        self.active_visualizer().update(audio_frame.clone());
    }

    pub fn render_to_texture(&mut self, gl: &gfx::gl::Gl) -> u32 {
        self.active_visualizer().render_to_texture(gl);

        self.texture_id
    }

    fn active_visualizer(&mut self) -> &mut SubVisualizer {
        match self.selected_visualizer.as_ref() {
            "equalizer" => &mut self.equalizer_visualizer,
            "power_circles" => &mut self.power_circles_visualizer,
            "smiley" => &mut self.smiley_visualizer,
            "bisymmetry" => &mut self.bisymmetry_visualizer,
            "trisymmetry" => &mut self.trisymmetry_visualizer,
            "quadsymmetry" => &mut self.quadsymmetry_visualizer,
            "pentasymmetry" => &mut self.pentasymmetry_visualizer,

            _ => &mut self.equalizer_visualizer,
        }
    }
}

impl SubVisualizer {
    fn setup(&self, gl: &gfx::gl::Gl, framebuffer_id: u32) -> (u32, u32) {
        unsafe {
            let vs = gl_try!(gl; gl.CreateShader(gl::VERTEX_SHADER));
            gl_try!(gl; gl.ShaderSource(vs, 1, [self.vs_src().as_ptr() as *const _].as_ptr(), ptr::null()));
            gl_try!(gl; gl.CompileShader(vs));

            let mut is_compiled = mem::uninitialized();
            gl_try!(gl; gl.GetShaderiv(vs, gl::COMPILE_STATUS, &mut is_compiled));
            if is_compiled == gl::FALSE as i32 {
                let mut max_length = mem::uninitialized();
                gl_try!(gl; gl.GetShaderiv(vs, gl::INFO_LOG_LENGTH, &mut max_length));

                let mut info_log = vec![0; max_length as usize];
                gl_try!(gl; gl.GetShaderInfoLog(vs, max_length, &mut max_length, info_log.as_mut_ptr()));

                for info_char in info_log.iter() {
                    print!("{}", *info_char as u8 as char);
                }
                panic!();
            }

            let fs = gl_try!(gl; gl.CreateShader(gl::FRAGMENT_SHADER));
            gl_try!(gl; gl.ShaderSource(fs, 1, [self.fs_src().as_ptr() as *const _].as_ptr(), ptr::null()));
            gl_try!(gl; gl.CompileShader(fs));

            let mut is_compiled = mem::uninitialized();
            gl_try!(gl; gl.GetShaderiv(fs, gl::COMPILE_STATUS, &mut is_compiled));
            if is_compiled == gl::FALSE as i32 {
                let mut max_length = mem::uninitialized();
                gl_try!(gl; gl.GetShaderiv(fs, gl::INFO_LOG_LENGTH, &mut max_length));

                let mut info_log = vec![0; max_length as usize];
                gl_try!(gl; gl.GetShaderInfoLog(fs, max_length, &mut max_length, info_log.as_mut_ptr()));

                for info_char in info_log.iter() {
                    print!("{}", *info_char as u8 as char);
                }
                panic!();
            }

            let program_id = gl_try!(gl; gl.CreateProgram());
            gl_try!(gl; gl.AttachShader(program_id, vs));
            gl_try!(gl; gl.AttachShader(program_id, fs));
            gl_try!(gl; gl.LinkProgram(program_id));

            let mut is_linked = mem::uninitialized();
            gl_try!(gl; gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut is_linked));
            if is_linked == gl::FALSE as i32 {
                let mut max_length = mem::uninitialized();
                gl_try!(gl; gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut max_length));

                let mut info_log = vec![0; max_length as usize];
                gl_try!(gl; gl.GetProgramInfoLog(program_id, max_length, &mut max_length, info_log.as_mut_ptr()));

                for info_char in info_log.iter() {
                    print!("{}", *info_char as u8 as char);
                }
                panic!();
            }

            (program_id, framebuffer_id)
        }
    }
}
