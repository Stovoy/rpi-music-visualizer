use audio;
use gfx;
use gfx::gl;
use std::mem;
use std::ptr;
use visualizer::visualizer::SubVisualizer;

const NUM_SQUARES: usize = 1;
const NUM_VERTICIES_PER_SQUARE: usize = 6;

pub struct SmileyVisualizer {
    program_id: u32,
    framebuffer_id: u32,
    vertex_data: Vec<f32>,

    amplitude: f32,
    phase: f32,
}

impl SubVisualizer for SmileyVisualizer {
    fn new() -> SmileyVisualizer {
        SmileyVisualizer {
            program_id: 0,
            framebuffer_id: 0,
            vertex_data: Vec::new(),

            amplitude: 0.0,
            phase: 0.0,
        }
    }

    fn post_setup(&mut self, program_id: u32, framebuffer_id: u32) {
        self.program_id = program_id;
        self.framebuffer_id = framebuffer_id;
    }

    fn update(&mut self, audio_frame: audio::AudioFrame) {
        self.vertex_data = generate_vertex_data();

        // Sum the 1000-2000hz amplitudes.
        self.amplitude = 0.0;
        for i in 4..20 {
            self.amplitude += audio_frame.hundred_hz_buckets[i];
        }
        self.amplitude /= 1.0;
        self.amplitude = f32::min(1.0, self.amplitude);

        self.phase += 0.1;
        if self.phase >= 3.14 * 2.0 {
            self.phase -= 3.14 * 2.0;
        }
    }

    fn render_to_texture(&self, gl: &gfx::gl::Gl) {
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

            let amplitude_uniform = gl_try!(gl; gl.GetUniformLocation(self.program_id, b"amplitude\0".as_ptr() as *const _));
            gl_try!(gl; gl.Uniform1f(amplitude_uniform, self.amplitude));

            let phase_uniform = gl_try!(gl; gl.GetUniformLocation(self.program_id, b"phase\0".as_ptr() as *const _));
            gl_try!(gl; gl.Uniform1f(phase_uniform, self.phase));

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

    fn vs_src(&self) -> &[u8] {
        b"
#version 100
precision mediump float;

#define PI 3.1415926535897932384626433832795

uniform float amplitude;
uniform float phase;

attribute vec2 position;

// Variables for the Fragment Shader.
varying vec2 v_position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_position = position;
}
\0"
    }

    fn fs_src(&self) -> &[u8] {b"
#version 100
precision mediump float;

#define PI 3.1415926535897932384626433832795

uniform float amplitude;
uniform float phase;

// Interpolated from the Vertex Shader.
varying vec2 v_position;

bool in_circle(vec2 p, vec2 center, float radius) {
    float d_x = p.x - center.x;
    float d_y = p.y - center.y;
    return d_x * d_x + d_y * d_y < radius * radius;
}

float y_mouth_top() {
    float a = -amplitude * 3.0;
    float b = 0.0;
    float x_offset = 0.0;
    float y_offset = -0.25 + amplitude / 4.0;
    float x = v_position.x + x_offset;

    return a * (x * x) + b * x + y_offset;
}

float y_mouth_bottom() {
    float a = amplitude * 3.0;
    float b = 0.0;
    float x_offset = 0.0;
    float y_offset = -0.25 - amplitude / 4.0;
    float x = v_position.x + x_offset;

    return a * (x * x) + b * x + y_offset;
}

void main() {
    float border_radius = 0.82;

    vec2 eye_position_1 = vec2(-0.3, 0.25);
    float eye_radius_1 = 0.08 + sin(phase) * 0.02 + 0.02;

    vec2 eye_position_2 = vec2(0.3, 0.25);
    float eye_radius_2 = 0.08 + sin(phase) * 0.02 + 0.02;

    vec3 yellow = vec3(1.0, 1.0, 0.0);

    float mouth_width = 0.58;
    float epsilon = 0.025;

    if (!in_circle(v_position, vec2(0), border_radius)) {
        // Border.
        gl_FragColor = vec4(yellow, amplitude);
    } else if (in_circle(v_position, eye_position_1, eye_radius_1)) {
        // Eye 1.
        gl_FragColor = vec4(yellow, 1.0);
    } else if (in_circle(v_position, eye_position_2, eye_radius_2)) {
        // Eye 2.
        gl_FragColor = vec4(yellow, 1.0);
    } else if (v_position.x >= -mouth_width / 2.0 && v_position.x <= mouth_width / 2.0) {
        float y_top = y_mouth_top();
        float y_bottom = y_mouth_bottom();
        if (v_position.y <= y_top + epsilon && v_position.y >= y_top - epsilon) {
            // Mouth top.
            gl_FragColor = vec4(yellow, 1.0);
        } else if (v_position.y <= y_bottom + epsilon && v_position.y >= y_bottom - epsilon) {
            // Mouth bottom.
            gl_FragColor = vec4(yellow, 1.0);
        } else {
            gl_FragColor = vec4(0.0);
        }
    } else {
        gl_FragColor = vec4(0.0);
    }
}
\0"
    }
}

fn generate_vertex_data() -> Vec<f32> {
    let size = 1.0;

    vec![
        -size, -size,
        -size, size,
        size, size,
        -size, -size,
        size, -size,
        size, size,
    ]
}
