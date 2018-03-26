use std::f32;
use std::mem;
use std::ptr;
use glutin::{self, GlContext};


mod gl {
    pub use self::Gles2 as Gl;
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
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

#[derive(Clone)]
pub struct Gl {
    gl: gl::Gl
}

pub fn load(gl_window: &glutin::GlWindow) -> Gl {
    let gl = gl::Gl::load_with(|ptr| gl_window.get_proc_address(ptr) as *const _);

    unsafe {
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl.Disable(gl::DEPTH_TEST);
    }
    Gl { gl }
}

impl Gl {
    pub fn draw_frame(&mut self) {
        unsafe {
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.update_scene();
            self.gl.DrawArrays(gl::TRIANGLES, 0, 6 * NUM_PIXELS as i32);
        }
    }

    fn update_scene(&self) {
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

            let vertex_data = generate_vertex_data();

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

            let center_attrib = self.gl.GetAttribLocation(program, b"center\0".as_ptr() as *const _);
            let pos_attrib = self.gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let radius_attrib = self.gl.GetAttribLocation(program, b"radius\0".as_ptr() as *const _);
            let color_attrib = self.gl.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            self.gl.VertexAttribPointer(center_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                                        8 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        ptr::null());
            self.gl.VertexAttribPointer(pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                                        8 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (2 * mem::size_of::<f32>()) as *const () as *const _);
            self.gl.VertexAttribPointer(radius_attrib as gl::types::GLuint, 1, gl::FLOAT, 0,
                                        8 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (4 * mem::size_of::<f32>()) as *const () as *const _);
            self.gl.VertexAttribPointer(color_attrib as gl::types::GLuint, 3, gl::FLOAT, 0,
                                        8 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (5 * mem::size_of::<f32>()) as *const () as *const _);
            self.gl.EnableVertexAttribArray(center_attrib as gl::types::GLuint);
            self.gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            self.gl.EnableVertexAttribArray(radius_attrib as gl::types::GLuint);
            self.gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);
        }
    }
}

const FLOATS_PER_TRIANGLE: usize = 8;
const FLOATS_PER_PIXEL: usize = 6 * FLOATS_PER_TRIANGLE;

fn generate_vertex_data() -> [f32; FLOATS_PER_PIXEL * NUM_PIXELS] {
    let r = 1.0;
    let g = 1.0;
    let b = 1.0;

    let mut pixels_from_triangles: [f32; FLOATS_PER_PIXEL * NUM_PIXELS] = [0.0; FLOATS_PER_PIXEL * NUM_PIXELS];
    for pixel in 0..NUM_PIXELS {
        let (x, y) = get_pixel_real_position(pixel as u8);

        // 2 triangles per pixel to form a square around it.
        // Triangle 1
        // -, -
        let mut vertex_index = pixel * FLOATS_PER_PIXEL;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 5) as usize] = r;
        pixels_from_triangles[(vertex_index + 6) as usize] = g;
        pixels_from_triangles[(vertex_index + 7) as usize] = b;

        // -, +
        vertex_index += FLOATS_PER_TRIANGLE;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y + PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 5) as usize] = r;
        pixels_from_triangles[(vertex_index + 6) as usize] = g;
        pixels_from_triangles[(vertex_index + 7) as usize] = b;

        // +, +
        vertex_index += FLOATS_PER_TRIANGLE;
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
        vertex_index += FLOATS_PER_TRIANGLE;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 5) as usize] = r;
        pixels_from_triangles[(vertex_index + 6) as usize] = g;
        pixels_from_triangles[(vertex_index + 7) as usize] = b;

        // +, -
        vertex_index += FLOATS_PER_TRIANGLE;
        pixels_from_triangles[(vertex_index + 0) as usize] = x;
        pixels_from_triangles[(vertex_index + 1) as usize] = y;
        pixels_from_triangles[(vertex_index + 2) as usize] = x + PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 3) as usize] = y - PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 4) as usize] = PIXEL_RADIUS;
        pixels_from_triangles[(vertex_index + 5) as usize] = r;
        pixels_from_triangles[(vertex_index + 6) as usize] = g;
        pixels_from_triangles[(vertex_index + 7) as usize] = b;

        // +, +
        vertex_index += FLOATS_PER_TRIANGLE;
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

// Interpolated from the Fragment Shader.
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
        gl_FragColor = vec4(v_color * position_to_radius_ratio, 1.0);
    }
}
\0";
