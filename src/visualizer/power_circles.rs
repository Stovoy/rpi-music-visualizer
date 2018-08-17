use audio;
use gfx;
use gfx::gl;
use std::mem;
use std::ptr;

const NUM_SQUARES: usize = 4;
const NUM_VERTICIES_PER_SQUARE: usize = 6;
const NUM_ATTRIBUTES_PER_VERTEX: usize = 7;
const NUM_FLOATS: usize = NUM_SQUARES * NUM_VERTICIES_PER_SQUARE * NUM_ATTRIBUTES_PER_VERTEX;

pub struct PowerCirclesVisualizer {
    program_id: u32,
    framebuffer_id: u32,
    vertex_data: [f32; NUM_FLOATS],
}

impl PowerCirclesVisualizer {
    pub fn new() -> PowerCirclesVisualizer {
        PowerCirclesVisualizer {
            program_id: 0,
            framebuffer_id: 0,
            vertex_data: [0.0; NUM_FLOATS],
        }
    }

    pub fn setup(&mut self, gl: &gfx::gl::Gl, framebuffer_id: u32) {
        unsafe {
            let vs = gl_try!(gl; gl.CreateShader(gl::VERTEX_SHADER));
            gl_try!(gl; gl.ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null()));
            gl_try!(gl; gl.CompileShader(vs));

            let fs = gl_try!(gl; gl.CreateShader(gl::FRAGMENT_SHADER));
            gl_try!(gl; gl.ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), ptr::null()));
            gl_try!(gl; gl.CompileShader(fs));

            let program = gl_try!(gl; gl.CreateProgram());
            gl_try!(gl; gl.AttachShader(program, vs));
            gl_try!(gl; gl.AttachShader(program, fs));
            gl_try!(gl; gl.LinkProgram(program));

            self.program_id = program;

            let mut is_linked = mem::uninitialized();
            gl_try!(gl; gl.GetProgramiv(program, gl::LINK_STATUS, &mut is_linked));
            if is_linked == gl::FALSE as i32 {
                let mut max_length = mem::uninitialized();
                gl_try!(gl; gl.GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut max_length));

                let mut info_log = vec![0; max_length as usize];
                gl_try!(gl; gl.GetProgramInfoLog(program, max_length, &mut max_length, info_log.as_mut_ptr()));

                for info_char in info_log.iter() {
                    print!("{}", *info_char as u8 as char);
                }
                panic!();
            }

            self.framebuffer_id = framebuffer_id;
        }
    }

    pub fn update(&mut self, audio_frame: audio::AudioFrame) {
        let low = audio_frame.low_power;
        let mid = audio_frame.mid_power;
        let high = audio_frame.high_power;

        self.vertex_data = generate_vertex_data(low, mid, high);
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

            if gl_try!(gl; gl.BindVertexArray.is_loaded()) {
                let mut vao = mem::uninitialized();
                gl_try!(gl; gl.GenVertexArrays(1, &mut vao));
                gl_try!(gl; gl.BindVertexArray(vao));
            }

            let pos_attrib = gl_try!(gl; gl.GetAttribLocation(self.program_id, b"position\0".as_ptr() as *const _));
            let color_attrib = gl_try!(gl; gl.GetAttribLocation(self.program_id, b"color\0".as_ptr() as *const _));
            let radius_attrib = gl_try!(gl; gl.GetAttribLocation(self.program_id, b"radius\0".as_ptr() as *const _));
            let power_attrib = gl_try!(gl; gl.GetAttribLocation(self.program_id, b"power\0".as_ptr() as *const _));
            gl_try!(gl; gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                7 * mem::size_of::<f32>() as gl::types::GLsizei,
                ptr::null(),
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint, 3, gl::FLOAT, 0,
                7 * mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                radius_attrib as gl::types::GLuint, 1, gl::FLOAT, 0,
                7 * mem::size_of::<f32>() as gl::types::GLsizei,
                (5 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                power_attrib as gl::types::GLuint, 1, gl::FLOAT, 0,
                7 * mem::size_of::<f32>() as gl::types::GLsizei,
                (6 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(radius_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(power_attrib as gl::types::GLuint));

            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_id));
            gl_try!(gl; gl.DrawBuffers(1, [gl::COLOR_ATTACHMENT0].as_ptr()));
            gl_try!(gl; gl.DrawArrays(gl::TRIANGLES, 0, 6 * 4));

            gl_try!(gl; gl.DeleteBuffers(1, &vb));
            gl_try!(gl; gl.DeleteVertexArrays(1, &vao));
        }
    }
}

fn generate_vertex_data(low: f32, mid: f32, high: f32) -> [f32; NUM_FLOATS] {
    let s1 = 1.0;
    let s2 = 0.75;
    let s3 = 0.5;
    let s4 = 0.25;

    let r = 1.0;
    let g = 1.0;
    let b = 1.0;

    [
        -s1, -s1, r, 0.0, 0.0, s1, low,
        -s1, s1, r, 0.0, 0.0, s1, low,
        s1, s1, r, 0.0, 0.0, s1, low,
        -s1, -s1, r, 0.0, 0.0, s1, low,
        s1, -s1, r, 0.0, 0.0, s1, low,
        s1, s1, r, 0.0, 0.0, s1, low,
        -s2, -s2, 0.0, g, 0.0, s2, mid,
        -s2, s2, 0.0, g, 0.0, s2, mid,
        s2, s2, 0.0, g, 0.0, s2, mid,
        -s2, -s2, 0.0, g, 0.0, s2, mid,
        s2, -s2, 0.0, g, 0.0, s2, mid,
        s2, s2, 0.0, g, 0.0, s2, mid,
        -s3, -s3, 0.0, 0.0, b, s3, high,
        -s3, s3, 0.0, 0.0, b, s3, high,
        s3, s3, 0.0, 0.0, b, s3, high,
        -s3, -s3, 0.0, 0.0, b, s3, high,
        s3, -s3, 0.0, 0.0, b, s3, high,
        s3, s3, 0.0, 0.0, b, s3, high,
        -s4, -s4, 0.0, 0.0, 0.0, s4, 0.0,
        -s4, s4, 0.0, 0.0, 0.0, s4, 0.0,
        s4, s4, 0.0, 0.0, 0.0, s4, 0.0,
        -s4, -s4, 0.0, 0.0, 0.0, s4, 0.0,
        s4, -s4, 0.0, 0.0, 0.0, s4, 0.0,
        s4, s4, 0.0, 0.0, 0.0, s4, 0.0,
    ]
}

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

attribute vec2 position;
attribute vec3 color;
attribute float radius;
attribute float power;

// Variables for the Fragment Shader.
varying vec2 v_position;
varying vec3 v_color;
varying float v_radius;
varying float v_power;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_position = position;
    v_color = color;
    v_radius = radius;
    v_power = power;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

#define PI 3.1415926535897932384626433832795

// Interpolated from the Vertex Shader.
varying vec2 v_position;
varying vec3 v_color;
varying float v_radius;
varying float v_power;

void main() {
    if ((v_position.x * v_position.x) + (v_position.y * v_position.y) > v_radius * v_radius) {
        // Out of bounds.
        gl_FragColor = vec4(0.0);
    } else {
        vec2 mirrored_position = v_position;

        float angle = v_power * PI;
        vec2 sector_start = vec2(0.0 + v_radius * sin(angle), -1.0 + v_radius * (1.0 - cos(angle)));
        vec2 sector_end = vec2(0.0, -1.0);

        if (mirrored_position.x > 0.0) {
            mirrored_position.x = -mirrored_position.x;
        }

        bool clockwise_from_start = (
            -sector_start.x * mirrored_position.y +
            sector_start.y * mirrored_position.x > 0.0
        );
        bool clockwise_from_end = (
            -sector_end.x * mirrored_position.y +
            sector_end.y * mirrored_position.x > 0.0
        );

        if (!clockwise_from_start && clockwise_from_end) {
            // In the sector.
            float y_scaling = (-v_position.y + 3.0) / (1.0 + 3.0);
            gl_FragColor = vec4(v_color * y_scaling, 1.0);
        } else {
            gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
}
\0";
