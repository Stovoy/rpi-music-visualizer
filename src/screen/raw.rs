use gfx;
use gfx::gl;
use screen;
use std::mem;
use std::ptr;

pub struct RawScreen {
    program_id: u32,
}

impl RawScreen {
    pub fn new() -> RawScreen {
        RawScreen { program_id: 0 }
    }
}

impl screen::Screen for RawScreen {
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

    fn render_from_texture(&mut self, gl: &gfx::gl::Gl, _texture: u32) {
        unsafe {
            gl_try!(gl; gl.UseProgram(self.program_id));

            let vertex_data: [f32; 2 * 6] = [
                -1.0, -1.0,
                1.0, -1.0,
                -1.0, 1.0,
                -1.0, 1.0,
                1.0, -1.0,
                1.0, 1.0
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

            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, 0));
            gl_try!(gl; gl.DrawArrays(gl::TRIANGLES, 0, 2 * 6));

            gl_try!(gl; gl.DeleteBuffers(1, &vb));
            gl_try!(gl; gl.DeleteVertexArrays(1, &vao));
        }
    }

    fn uses_window(&self) -> bool {
        true
    }
}

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

attribute vec2 position;

varying vec2 v_position;

void main(){
    gl_Position = vec4(position, 0.0, 1.0);
    v_position = position;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

uniform sampler2D texture_sampler;

// Interpolated from the Vertex Shader.
varying vec2 v_position;

void main() {
    float x = (v_position.x + 1.0) / 2.0;
    float y = (v_position.y + 1.0) / 2.0;
    gl_FragColor = texture2D(texture_sampler, vec2(x, y));
}
\0";
