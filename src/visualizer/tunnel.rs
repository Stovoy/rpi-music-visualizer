use std::ptr;
use std::mem;

use audio;
use gfx;
use gfx::gl;
use visualizer::shader_libs;

pub struct TunnelVisualizer {
    program_id: u32,
    framebuffer_id: u32,

    time: f32,

    low_power: f32,
    mid_power: f32,

    low_power_counter: f32,
}

impl TunnelVisualizer {
    pub fn new() -> TunnelVisualizer {
        TunnelVisualizer {
            program_id: 0,
            framebuffer_id: 0,

            time: 0.0,

            low_power: 0.0,
            mid_power: 0.0,

            low_power_counter: 0.0,
        }
    }

    pub fn setup(&mut self, gl: &gfx::gl::Gl, framebuffer_id: u32) {
        unsafe {
            let vs = gl_try!(gl; gl.CreateShader(gl::VERTEX_SHADER));
            gl_try!(gl; gl.ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null()));
            gl_try!(gl; gl.CompileShader(vs));

            let fs = gl_try!(gl; gl.CreateShader(gl::FRAGMENT_SHADER));

            let mut fs_src = Vec::new();
            fs_src.extend_from_slice(&shader_libs::HELPERS.as_bytes());
            fs_src.extend_from_slice(&VISUALIZER_IMPL.as_bytes());
            fs_src.extend_from_slice(&shader_libs::RAYMARCHER.as_bytes());

            // for fs_src_char in fs_src.iter() {
            //     print!("{}", *fs_src_char as u8 as char);
            // }

            gl_try!(gl; gl.ShaderSource(fs, 1, [fs_src.as_ptr() as *const _].as_ptr(), ptr::null()));
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

    pub fn render_to_texture(&self, gl: &gfx::gl::Gl) {
        unsafe {
            gl_try!(gl; gl.UseProgram(self.program_id));

            let vertex_data: [f32; 6 * 2] = [
                -1.0, -1.0,
                1.0, -1.0,
                -1.0, 1.0,
                -1.0, 1.0,
                1.0, -1.0,
                1.0, 1.0,
            ];

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

            let pos_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"position\0".as_ptr() as *const _));
            gl_try!(gl; gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                2 * mem::size_of::<f32>() as gl::types::GLsizei,
                ptr::null(),
            ));
            gl_try!(gl; gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint));

            let time = gl_try!(gl; gl.GetUniformLocation(
                self.program_id, b"time\0".as_ptr() as *const _));
            let mid_power = gl_try!(gl; gl.GetUniformLocation(
                self.program_id, b"midPower\0".as_ptr() as *const _));
            let low_power_counter = gl_try!(gl; gl.GetUniformLocation(
                self.program_id, b"lowPowerCounter\0".as_ptr() as *const _));

            gl_try!(gl; gl.Uniform1f(time, self.time));
            gl_try!(gl; gl.Uniform1f(mid_power, self.mid_power));
            gl_try!(gl; gl.Uniform1f(low_power_counter, self.low_power_counter));

            gl_try!(gl; gl.ClearColor(0.0, 0.0, 0.0, 1.0));
            gl_try!(gl; gl.Clear(gl::COLOR_BUFFER_BIT));

            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_id));
            gl_try!(gl; gl.DrawBuffers(1, [gl::COLOR_ATTACHMENT0].as_ptr()));
            gl_try!(gl; gl.DrawArrays(gl::TRIANGLES, 0, 6 * 2));
        }
    }

    pub fn update(&mut self, audio_frame: audio::AudioFrame) {
        // let low = audio_frame.low_power;
        self.mid_power = audio_frame.mid_power;
        // let high = audio_frame.high_power;

        self.low_power_counter += (audio_frame.low_power - self.low_power).abs() / 75.0 + 0.005;
        if self.low_power_counter >= 1.0 {
            self.low_power_counter = self.low_power_counter - 1.0;
        }

        self.low_power = audio_frame.low_power;

        self.time += 0.1;

    }
}

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

attribute vec2 position;

// Variables for the Fragment Shader.
varying vec2 v_position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_position = position;
}
\0";

const VISUALIZER_IMPL: &'static str = include_str!("shaders/tunnel.glsl");
