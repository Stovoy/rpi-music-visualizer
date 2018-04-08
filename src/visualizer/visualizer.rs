use std::ptr;
use std::mem;

use audio;
use gfx;
use gfx::gl;

pub struct Visualizer {
    framebuffer_id: u32,
    program_id: u32,
    texture_id: u32,
}

impl Visualizer {
    pub fn new() -> Visualizer {
        Visualizer {
            framebuffer_id: 0,
            program_id: 0,
            texture_id: 0,
        }
    }

    pub fn setup(&mut self, gl: &gfx::gl::Gl) {
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

                let mut info_log = vec![0 as i8; max_length as usize];
                gl_try!(gl; gl.GetProgramInfoLog(program, max_length, &mut max_length, info_log.as_mut_ptr()));

                for info_char in info_log.iter() {
                    print!("{}", *info_char as u8 as char);
                }
                panic!();
            }

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
        }
    }

    pub fn render_to_texture(&self, gl: &gfx::gl::Gl, audio_frame: audio::AudioFrame) -> u32 {
        let low = audio_frame.low_power;
        let mid = audio_frame.mid_power;
        let high = audio_frame.high_power;

        unsafe {
            gl_try!(gl; gl.UseProgram(self.program_id));

            let vertex_data = generate_vertex_data(low, mid, high);

            let mut vb = mem::uninitialized();
            gl_try!(gl; gl.GenBuffers(1, &mut vb));
            gl_try!(gl; gl.BindBuffer(gl::ARRAY_BUFFER, vb));
            gl_try!(gl; gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertex_data.as_ptr() as *const _,
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
        }

        self.texture_id
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
