use std::f32;
use std::mem;
use std::ptr;
use gl;

use screen;
use gfx;

pub struct LedDiskEmulatorScreen {
    program_id: u32,
}

impl LedDiskEmulatorScreen {
    pub fn new() -> LedDiskEmulatorScreen {
        LedDiskEmulatorScreen { program_id: 0 }
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

                let mut info_log = vec![0 as i8; max_length as usize];
                gl_try!(gl; gl.GetProgramInfoLog(program, max_length, &mut max_length, info_log.as_mut_ptr()));

                for info_char in info_log.iter() {
                    print!("{}", *info_char as u8 as char);
                }
                panic!();
            }
        }
    }

    fn render_from_texture(&self, gl: &gfx::gl::Gl, _texture: u32) {
        unsafe {
            gl_try!(gl; gl.UseProgram(self.program_id));

            let vertex_data = generate_vertex_data();

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

            let center_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"center\0".as_ptr() as *const _));
            let pos_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"position\0".as_ptr() as *const _));
            let radius_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"radius\0".as_ptr() as *const _));
            gl_try!(gl; gl.VertexAttribPointer(
                center_attrib as gl::types::GLuint, 2, gl::FLOAT,
                0, 5 * mem::size_of::<f32>() as gl::types::GLsizei,
                ptr::null(),
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint, 2, gl::FLOAT,
                0, 5 * mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                radius_attrib as gl::types::GLuint, 1, gl::FLOAT,
                0, 5 * mem::size_of::<f32>() as gl::types::GLsizei,
                (4 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.EnableVertexAttribArray(center_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(radius_attrib as gl::types::GLuint));

            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, 0));

            gl_try!(gl; gl.ClearColor(0.0, 0.0, 0.0, 1.0));
            gl_try!(gl; gl.Clear(gl::COLOR_BUFFER_BIT));

            gl_try!(gl; gl.DrawArrays(gl::TRIANGLES, 0,
                FLOATS_PER_PIXEL as i32 * NUM_PIXELS as i32));
        }
    }
}


static RINGS: [[u8; 2]; 10] = [
    [254, 254],
    [248, 253],
    [236, 247],
    [216, 235],
    [192, 215],
    [164, 191],
    [132, 163],
    [92, 131],
    [48, 91],
    [0, 47],
];

const DISTANCE_BETWEEN_RINGS: f32 = 0.1;
const PIXEL_RADIUS: f32 = 0.03;

const NUM_PIXELS: usize = 255;

// Given a pixel from 0..255, return it's x y position as a 2d float in the gl point psace
// [-1, 1] * [-1..1]
fn get_pixel_real_position(pixel: u8) -> (f32, f32) {
    let ring_index = get_pixel_ring_index(pixel);
    let start_index = RINGS[ring_index as usize][0];
    let end_index = RINGS[ring_index as usize][1];
    let radians_between_pixels = 2.0 * f32::consts::PI / (end_index - start_index + 1) as f32;
    let angle = (pixel - start_index) as f32 * radians_between_pixels;
    let radius = ring_index as f32 * DISTANCE_BETWEEN_RINGS;

    (radius * f32::cos(angle), radius * f32::sin(angle))
}

// Get the ring index for given pixel.
fn get_pixel_ring_index(pixel: u8) -> u8 {
    for (i, ring) in RINGS.iter().enumerate() {
        if pixel >= ring[0] && pixel <= ring[1] {
            return i as u8;
        }
    }

    0
}


const FLOATS_PER_VERTEX: usize = 5;
const FLOATS_PER_PIXEL: usize = 6 * FLOATS_PER_VERTEX;

fn generate_vertex_data() -> [f32; FLOATS_PER_PIXEL * NUM_PIXELS] {
    let mut pixels_from_triangles: [f32; FLOATS_PER_PIXEL * NUM_PIXELS] =
        [0.0; FLOATS_PER_PIXEL * NUM_PIXELS];
    for pixel in 0..NUM_PIXELS {
        let (x, y) = get_pixel_real_position(pixel as u8);

        // 2 triangles per pixel to form a square enclosing it.
        // Triangle 1
        // -, -
        let mut vertex_index = pixel * FLOATS_PER_PIXEL;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;

        // -, +
        vertex_index += FLOATS_PER_VERTEX;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y + PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;

        // +, +
        vertex_index += FLOATS_PER_VERTEX;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x + PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y + PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;

        // Triangle 2
        // -, -
        vertex_index += FLOATS_PER_VERTEX;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;

        // +, -
        vertex_index += FLOATS_PER_VERTEX;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x + PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;

        // +, +
        vertex_index += FLOATS_PER_VERTEX;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x + PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y + PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
    }

    pixels_from_triangles
}

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

attribute vec2 center;
attribute vec2 position;
attribute float radius;

// Variables for the Fragment Shader.
varying vec2 v_center;
varying vec2 v_position;
varying float v_radius;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_center = center;
    v_position = position;
    v_radius = radius;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

// Interpolated from the Fragment Shader.
varying vec2 v_center;
varying vec2 v_position;
varying float v_radius;

uniform sampler2D texture_sampler;

void main() {
    vec2 rel_pos = v_position - v_center;
    float center_dist = (rel_pos.x * rel_pos.x) + (rel_pos.y * rel_pos.y);
    float radius_dist = v_radius * v_radius;
    if (center_dist > radius_dist) {
        // Out of bounds.
        gl_FragColor = vec4(0.0);
    } else {
        float x = (v_position.x + 1.0) / 2.0;
        float y = (v_position.y + 1.0) / 2.0;
        vec4 color = texture2D(texture_sampler, vec2(x, y));

        float position_to_radius_ratio = 1.0 - (center_dist / radius_dist);
        color = color * position_to_radius_ratio;

        gl_FragColor = color;
    }
}
\0";
