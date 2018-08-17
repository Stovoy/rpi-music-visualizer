use std::f32;
use std::mem;
use std::ptr;
use gl;

use led_mapper;
use led_mapper::led_disk_mapper::NUM_PIXELS;
use led_mapper::led_disk_mapper::PIXEL_RADIUS;
use screen;
use gfx;

pub struct LedDiskEmulatorScreen {
    program_id: u32,
    mapper: led_mapper::LedDiskMapper,
}

const FLOATS_PER_VERTEX: usize = 8;
const FLOATS_PER_PIXEL: usize = 6 * FLOATS_PER_VERTEX;

impl LedDiskEmulatorScreen {
    pub fn new() -> LedDiskEmulatorScreen {
        LedDiskEmulatorScreen {
            program_id: 0,
            mapper: led_mapper::LedDiskMapper::new(),
        }
    }

    fn generate_vertex_data(&self, pixel_colors: [(u8, u8, u8); NUM_PIXELS]) -> [f32; FLOATS_PER_PIXEL * NUM_PIXELS] {
        let mut pixels_from_triangles: [f32; FLOATS_PER_PIXEL * NUM_PIXELS] =
            [0.0; FLOATS_PER_PIXEL * NUM_PIXELS];
        for pixel_index in 0..NUM_PIXELS {
            let (x, y) = self.mapper.get_pixel_normalized_position(pixel_index as u8);

            let r = pixel_colors[pixel_index].0 as f32 / 255.0;
            let g = pixel_colors[pixel_index].1 as f32 / 255.0;
            let b = pixel_colors[pixel_index].2 as f32 / 255.0;

            // 2 triangles per pixel to form a square enclosing it.
            // Triangle 1
            // -, -
            let mut vertex_index = pixel_index * FLOATS_PER_PIXEL;
            pixels_from_triangles[(vertex_index + 0) as usize] = x;
            pixels_from_triangles[(vertex_index + 1) as usize] = y;
            pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 5) as usize] = r;
            pixels_from_triangles[(vertex_index + 6) as usize] = g;
            pixels_from_triangles[(vertex_index + 7) as usize] = b;

            // -, +
            vertex_index += FLOATS_PER_VERTEX;
            pixels_from_triangles[(vertex_index + 0) as usize] = x;
            pixels_from_triangles[(vertex_index + 1) as usize] = y;
            pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 3) as usize] = y + PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 5) as usize] = r;
            pixels_from_triangles[(vertex_index + 6) as usize] = g;
            pixels_from_triangles[(vertex_index + 7) as usize] = b;

            // +, +
            vertex_index += FLOATS_PER_VERTEX;
            pixels_from_triangles[(vertex_index + 0) as usize] = x;
            pixels_from_triangles[(vertex_index + 1) as usize] = y;
            pixels_from_triangles[(vertex_index + 2) as usize] = x + PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 3) as usize] = y + PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 5) as usize] = r;
            pixels_from_triangles[(vertex_index + 6) as usize] = g;
            pixels_from_triangles[(vertex_index + 7) as usize] = b;

            // Triangle 2
            // -, -
            vertex_index += FLOATS_PER_VERTEX;
            pixels_from_triangles[(vertex_index + 0) as usize] = x;
            pixels_from_triangles[(vertex_index + 1) as usize] = y;
            pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 5) as usize] = r;
            pixels_from_triangles[(vertex_index + 6) as usize] = g;
            pixels_from_triangles[(vertex_index + 7) as usize] = b;

            // +, -
            vertex_index += FLOATS_PER_VERTEX;
            pixels_from_triangles[(vertex_index + 0) as usize] = x;
            pixels_from_triangles[(vertex_index + 1) as usize] = y;
            pixels_from_triangles[(vertex_index + 2) as usize] = x + PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 5) as usize] = r;
            pixels_from_triangles[(vertex_index + 6) as usize] = g;
            pixels_from_triangles[(vertex_index + 7) as usize] = b;

            // +, +
            vertex_index += FLOATS_PER_VERTEX;
            pixels_from_triangles[(vertex_index + 0) as usize] = x;
            pixels_from_triangles[(vertex_index + 1) as usize] = y;
            pixels_from_triangles[(vertex_index + 2) as usize] = x + PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 3) as usize] = y + PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
            pixels_from_triangles[(vertex_index + 5) as usize] = r;
            pixels_from_triangles[(vertex_index + 6) as usize] = g;
            pixels_from_triangles[(vertex_index + 7) as usize] = b;
        }

        pixels_from_triangles
    }
}

impl screen::Screen for LedDiskEmulatorScreen {
    fn setup(&mut self, gl: &gfx::gl::Gl) {
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
        }
    }

    fn render_from_texture(&mut self, gl: &gfx::gl::Gl, texture: u32) {
        unsafe {
            let pixel_colors = self.mapper.map_from_texture(gl, texture);

            gl_try!(gl; gl.UseProgram(self.program_id));

            let vertex_data = self.generate_vertex_data(pixel_colors);

            let mut vb = mem::uninitialized();
            gl_try!(gl; gl.GenBuffers(1, &mut vb));
            gl_try!(gl; gl.BindBuffer(gl::ARRAY_BUFFER, vb));
            gl_try!(gl; gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            ));

            let mut vao = mem::uninitialized();
            gl_try!(gl; gl.GenVertexArrays(1, &mut vao));
            gl_try!(gl; gl.BindVertexArray(vao));

            let center_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"center\0".as_ptr() as *const _));
            let pos_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"position\0".as_ptr() as *const _));
            let radius_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"radius\0".as_ptr() as *const _));
            let color_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"color\0".as_ptr() as *const _));
            gl_try!(gl; gl.VertexAttribPointer(
                center_attrib as gl::types::GLuint, 2, gl::FLOAT,
                0, 8 * mem::size_of::<f32>() as gl::types::GLsizei,
                ptr::null(),
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint, 2, gl::FLOAT,
                0, 8 * mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                radius_attrib as gl::types::GLuint, 1, gl::FLOAT,
                0, 8 * mem::size_of::<f32>() as gl::types::GLsizei,
                (4 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint, 3, gl::FLOAT,
                0, 8 * mem::size_of::<f32>() as gl::types::GLsizei,
                (5 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.EnableVertexAttribArray(center_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(radius_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint));

            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, 0));

            gl_try!(gl; gl.ClearColor(0.0, 0.0, 0.0, 1.0));
            gl_try!(gl; gl.Clear(gl::COLOR_BUFFER_BIT));

            gl_try!(gl; gl.DrawArrays(gl::TRIANGLES, 0,
                FLOATS_PER_PIXEL as i32 * NUM_PIXELS as i32));

            gl_try!(gl; gl.DeleteBuffers(1, &vb));gl_try!(gl; gl.DeleteVertexArrays(1, &vao));
        }
    }

    fn uses_window(&self) -> bool {
        true
    }
}


const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

attribute vec2 center;
attribute vec2 position;
attribute float radius;
attribute vec3 color;

// Variables for the Fragment Shader.
varying vec2 v_center;
varying vec2 v_position;
varying float v_radius;
varying vec3 v_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_center = center;
    v_position = position;
    v_radius = radius;
    v_color = color;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

// Interpolated from the Vertex Shader.
varying vec2 v_center;
varying vec2 v_position;
varying float v_radius;
varying vec3 v_color;

void main() {
    vec2 rel_pos = v_position - v_center;
    float center_dist = (rel_pos.x * rel_pos.x) + (rel_pos.y * rel_pos.y);
    float radius_dist = v_radius * v_radius;
    if (center_dist > radius_dist) {
        // Out of bounds.
        gl_FragColor = vec4(0.0);
    } else {
        float position_to_radius_ratio = 1.0 - (center_dist / radius_dist);

        vec3 color = v_color;
        if (position_to_radius_ratio < 0.8) {
            color *= position_to_radius_ratio * 0.75;
        }

        gl_FragColor = vec4(color, 1.0);
    }
}
\0";
