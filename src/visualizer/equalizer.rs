use audio;
use gfx;
use gfx::gl;
use std::mem;
use std::ptr;

const NUM_SQUARES: usize = 7;
const NUM_VERTICIES_PER_SQUARE: usize = 6;
const NUM_ATTRIBUTES_PER_VERTEX: usize = 7;
const NUM_FLOATS: usize = NUM_SQUARES * NUM_VERTICIES_PER_SQUARE * NUM_ATTRIBUTES_PER_VERTEX;

pub struct EqualizerVisualizer {
    program_id: u32,
    framebuffer_id: u32,
    vertex_data: Vec<f32>,

    phase: f32,
}

impl EqualizerVisualizer {
    pub fn new() -> EqualizerVisualizer {
        EqualizerVisualizer {
            program_id: 0,
            framebuffer_id: 0,
            vertex_data: Vec::new(),

            phase: 0.0,
        }
    }

    pub fn setup(&mut self, gl: &gfx::gl::Gl, framebuffer_id: u32) {
        unsafe {
            let vs = gl_try!(gl; gl.CreateShader(gl::VERTEX_SHADER));
            gl_try!(gl; gl.ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null()));
            gl_try!(gl; gl.CompileShader(vs));

            let mut is_compiled = mem::uninitialized();
            gl_try!(gl; gl.GetShaderiv(vs, gl::COMPILE_STATUS, &mut is_compiled));
            if is_compiled == gl::FALSE as i32 {
                let mut max_length = mem::uninitialized();
                gl_try!(gl; gl.GetShaderiv(vs, gl::INFO_LOG_LENGTH, &mut max_length));

                let mut info_log = vec![0 as i8; max_length as usize];
                gl_try!(gl; gl.GetShaderInfoLog(vs, max_length, &mut max_length, info_log.as_mut_ptr()));

                for info_char in info_log.iter() {
                    print!("{}", *info_char as u8 as char);
                }
                panic!();
            }

            let fs = gl_try!(gl; gl.CreateShader(gl::FRAGMENT_SHADER));
            gl_try!(gl; gl.ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), ptr::null()));
            gl_try!(gl; gl.CompileShader(fs));

            let mut is_compiled = mem::uninitialized();
            gl_try!(gl; gl.GetShaderiv(fs, gl::COMPILE_STATUS, &mut is_compiled));
            if is_compiled == gl::FALSE as i32 {
                let mut max_length = mem::uninitialized();
                gl_try!(gl; gl.GetShaderiv(fs, gl::INFO_LOG_LENGTH, &mut max_length));

                let mut info_log = vec![0 as i8; max_length as usize];
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

                let mut info_log = vec![0 as i8; max_length as usize];
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
        self.vertex_data = generate_vertex_data(audio_frame);
        self.phase += 0.1;
        if self.phase >= 3.14 * 2.0 {
            self.phase -= 3.14 * 2.0;
        }
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

            let phase_uniform = gl_try!(gl; gl.GetUniformLocation(self.program_id, b"phase\0".as_ptr() as *const _));
            gl_try!(gl; gl.Uniform1f(phase_uniform, self.phase));

            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_id));

            gl_try!(gl; gl.ClearColor(0.0, 0.0, 0.0, 1.0));
            gl_try!(gl; gl.Clear(gl::COLOR_BUFFER_BIT));

            gl_try!(gl; gl.DrawBuffers(1, [gl::COLOR_ATTACHMENT0].as_ptr()));

            gl_try!(gl; gl.DrawArrays(gl::TRIANGLES, 0,
                (NUM_SQUARES * NUM_VERTICIES_PER_SQUARE) as i32));
        }
    }
}

fn generate_vertex_data(audio_frame: audio::AudioFrame) -> Vec<f32> {
    let square_sizes = [
        1.0,
        1.0 - 1.0 / 7.0,
        1.0 - 2.0 / 7.0,
        1.0 - 3.0 / 7.0,
        1.0 - 4.0 / 7.0,
        1.0 - 5.0 / 7.0,
        1.0 - 6.0 / 7.0,
    ];

    let square_colors = [
        (148, 0, 211),
        (75, 0, 130),
        (0, 0, 255),
        (0, 255, 0),
        (255, 255, 0),
        (255, 127, 0),
        (255, 0, 0),
    ];

    let mut vertex_data = Vec::with_capacity(NUM_FLOATS);

    let add_square = |vertex_data: &mut Vec<f32>,
                          size: f32, color: (f32, f32, f32), amplitude: f32| {
        vertex_data.extend_from_slice(&[
            -size, -size, color.0, color.1, color.2, size, amplitude,
            -size, size, color.0, color.1, color.2, size, amplitude,
            size, size, color.0, color.1, color.2, size, amplitude,
            -size, -size, color.0, color.1, color.2, size, amplitude,
            size, -size, color.0, color.1, color.2, size, amplitude,
            size, size, color.0, color.1, color.2, size, amplitude,
        ]);
    };

    let buckets_per_square = (
        6 as f32 / NUM_SQUARES as f32
    ).round() as usize;
    for i in 0..NUM_SQUARES {
        let mut amplitude = 0.0;
        for j in 0..buckets_per_square {
            let index = i * buckets_per_square + j;
            if index >= audio_frame.hundred_hz_buckets.len() {
                break;
            }
            amplitude += audio_frame.hundred_hz_buckets[NUM_SQUARES - index - 1];
        }
        amplitude = f32::min(1.0, amplitude);

        let color = (square_colors[i].0 as f32 / 255.0,
                     square_colors[i].1 as f32 / 255.0,
                     square_colors[i].2 as f32 / 255.0);
        add_square(&mut vertex_data, square_sizes[i], color, amplitude);
    }

    vertex_data
}

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

#define PI 3.1415926535897932384626433832795

uniform float phase;

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
    v_position = position;
    float x = sin((v_position.y + 2.0) * PI + phase) * radius / 1.0;
    float y = sin((v_position.x + 2.0) * PI + phase) * radius / 1.0;
    // if (v_position.x < 0.0) {
    //     v_position.x += x;
    // } else {
    //     v_position.x -= x;
    // }
    // if (v_position.y < 0.0) {
    //     v_position.y -= y;
    // } else {
    //     v_position.y += y;
    // }
    gl_Position = vec4(v_position, 0.0, 1.0);
    v_color = color;
    v_radius = radius;
    v_power = power;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

#define PI 3.1415926535897932384626433832795

uniform float phase;

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

        if (true || (!clockwise_from_start && clockwise_from_end)) {
            // In the sector.
            float y_scaling = (-v_position.y + 3.0) / (1.0 + 3.0);
            gl_FragColor = vec4(v_color * y_scaling * v_power, 1.0);
        } else {
            gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
}
\0";
