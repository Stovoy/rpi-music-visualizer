use std::mem;
use std::ptr;
use glutin::{self, GlContext};

use audio;

mod gl {
    pub use self::Gles2 as Gl;
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}


#[derive(Clone)]
pub struct Gl {
    gl: gl::Gl,
    frame: u32,
}

pub fn load(gl_window: &glutin::GlWindow) -> Gl {
    let gl = gl::Gl::load_with(|ptr| gl_window.get_proc_address(ptr) as *const _);

    unsafe {
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl.Disable(gl::DEPTH_TEST);
    }
    Gl { gl, frame: 0 }
}

impl Gl {
    pub fn draw_frame(&mut self, audio_frame: audio::AudioFrame) {
        unsafe {
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.update_scene(audio_frame.low_power, audio_frame.mid_power, audio_frame.high_power);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 6 * 4);
            self.frame += 1;
        }
    }

    fn update_scene(&self, low: f32, mid: f32, high: f32) {
        unsafe {
            let vs = self.gl.CreateShader(gl::VERTEX_SHADER);
            self.gl.ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
            self.gl.CompileShader(vs);

            let fs = self.gl.CreateShader(gl::FRAGMENT_SHADER);
            self.gl.ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
            self.gl.CompileShader(fs);

            let program = self.gl.CreateProgram();
            self.gl.AttachShader(program, vs);
            self.gl.AttachShader(program, fs);
            self.gl.LinkProgram(program);
            self.gl.UseProgram(program);

            let vertex_data = generate_vertex_data(low, mid, high);

            let mut vb = mem::uninitialized();
            self.gl.GenBuffers(1, &mut vb);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            self.gl.BufferData(gl::ARRAY_BUFFER,
                               (vertex_data.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                               vertex_data.as_ptr() as *const _, gl::STATIC_DRAW);

            if self.gl.BindVertexArray.is_loaded() {
                let mut vao = mem::uninitialized();
                self.gl.GenVertexArrays(1, &mut vao);
                self.gl.BindVertexArray(vao);
            }

            let pos_attrib = self.gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let color_attrib = self.gl.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            let radius_attrib = self.gl.GetAttribLocation(program, b"radius\0".as_ptr() as *const _);
            let power_attrib = self.gl.GetAttribLocation(program, b"power\0".as_ptr() as *const _);
            self.gl.VertexAttribPointer(pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                                        7 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        ptr::null());
            self.gl.VertexAttribPointer(color_attrib as gl::types::GLuint, 3, gl::FLOAT, 0,
                                        7 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (2 * mem::size_of::<f32>()) as *const () as *const _);
            self.gl.VertexAttribPointer(radius_attrib as gl::types::GLuint, 1, gl::FLOAT, 0,
                                        7 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (5 * mem::size_of::<f32>()) as *const () as *const _);
            self.gl.VertexAttribPointer(power_attrib as gl::types::GLuint, 1, gl::FLOAT, 0,
                                        7 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (6 * mem::size_of::<f32>()) as *const () as *const _);
            self.gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            self.gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);
            self.gl.EnableVertexAttribArray(radius_attrib as gl::types::GLuint);
            self.gl.EnableVertexAttribArray(power_attrib as gl::types::GLuint);
        }
    }
}

fn generate_vertex_data(low: f32, mid: f32, high: f32) -> [f32; 6 * 7 * 4] {
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

// Interpolated from the Fragment Shader.
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

        bool clockwise_from_start = -sector_start.x * mirrored_position.y + sector_start.y * mirrored_position.x > 0.0;
        bool clockwise_from_end = -sector_end.x * mirrored_position.y + sector_end.y * mirrored_position.x > 0.0;

        if (!clockwise_from_start && clockwise_from_end) {
            // In the sector.
            float y_scaling = (-v_position.y + 3.0) / (1.0 + 3.0);
            gl_FragColor = vec4(v_color * v_power * y_scaling, 1.0);
        } else {
            gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
}
\0";
