use audio;
use gfx;
use gfx::gl;
use std::mem;
use std::ptr;

const NUM_SQUARES: usize = 1;
const NUM_VERTICIES_PER_SQUARE: usize = 6;
const NUM_ATTRIBUTES_PER_VERTEX: usize = 2;
const NUM_FLOATS: usize = NUM_SQUARES * NUM_VERTICIES_PER_SQUARE * NUM_ATTRIBUTES_PER_VERTEX;

pub struct SymmetryVisualizer {
    program_id: u32,
    framebuffer_id: u32,
    vertex_data: Vec<f32>,

    phase: f32,
    speed: f32,
}

const MAX_SPEED: f32 = 0.2;

impl SymmetryVisualizer {
    pub fn new() -> SymmetryVisualizer {
        SymmetryVisualizer {
            program_id: 0,
            framebuffer_id: 0,
            vertex_data: generate_vertex_data(),

            phase: 0.0,
            speed: 0.0,
        }
    }

    pub fn post_setup(&mut self, program_id: u32, framebuffer_id: u32) {
        self.program_id = program_id;
        self.framebuffer_id = framebuffer_id;
    }

    pub fn update(&mut self, audio_frame: audio::AudioFrame) {
        let mut amplitude = 0.0;
        for i in 0..20 {
            if i >= audio_frame.hundred_hz_buckets.len() {
                break;
            }
            amplitude += audio_frame.hundred_hz_buckets[i];
        }
        amplitude /= 2.0;
        amplitude = f32::min(1.0, amplitude);

        self.speed = amplitude * MAX_SPEED;
        self.phase += self.speed;
    }

    pub fn render_to_texture(&self, gl: &gfx::gl::Gl) {
        unsafe {
            gl_try!(gl; gl.UseProgram(self.program_id));

            let mut vb = mem::uninitialized();
            gl_try!(gl; gl.GenBuffers(1, &mut vb));
            gl_try!(gl; gl.BindBuffer(gl::ARRAY_BUFFER, vb));
            gl_try!(gl; gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex_data.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            ));

            let mut vao = mem::uninitialized();
            gl_try!(gl; gl.GenVertexArrays(1, &mut vao));
            gl_try!(gl; gl.BindVertexArray(vao));

            let pos_attrib = gl_try!(gl; gl.GetAttribLocation(self.program_id, b"position\0".as_ptr() as *const _));
            gl_try!(gl; gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                2 * mem::size_of::<f32>() as gl::types::GLsizei,
                ptr::null(),
            ));
            gl_try!(gl; gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint));

            let phase_uniform = gl_try!(gl; gl.GetUniformLocation(self.program_id, b"phase\0".as_ptr() as *const _));
            gl_try!(gl; gl.Uniform1f(phase_uniform, self.phase));

            let speed_uniform = gl_try!(gl; gl.GetUniformLocation(self.program_id, b"speed\0".as_ptr() as *const _));
            gl_try!(gl; gl.Uniform1f(speed_uniform, self.speed / MAX_SPEED));

            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_id));

            gl_try!(gl; gl.ClearColor(0.0, 0.0, 0.0, 1.0));
            gl_try!(gl; gl.Clear(gl::COLOR_BUFFER_BIT));

            gl_try!(gl; gl.DrawBuffers(1, [gl::COLOR_ATTACHMENT0].as_ptr()));

            gl_try!(gl; gl.DrawArrays(gl::TRIANGLES, 0,
                (NUM_SQUARES * NUM_VERTICIES_PER_SQUARE) as i32));

            gl_try!(gl; gl.DeleteBuffers(1, &vb));
            gl_try!(gl; gl.DeleteVertexArrays(1, &vao));
        }
    }
}

fn generate_vertex_data() -> Vec<f32> {
    let mut vertex_data = Vec::with_capacity(NUM_FLOATS);

    let add_square = |vertex_data: &mut Vec<f32>,
                      size: f32| {
        vertex_data.extend_from_slice(&[
            -size, -size,
            -size, size,
            size, size,
            -size, -size,
            size, -size,
            size, size,
        ]);
    };

    add_square(&mut vertex_data, 1.0);

    vertex_data
}
