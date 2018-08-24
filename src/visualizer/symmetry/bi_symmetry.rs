use audio;
use gfx;
use visualizer::symmetry::symmetry::SymmetryVisualizer;
use visualizer::visualizer::SubVisualizer;

pub struct BiSymmetryVisualizer {
    symmetry_visualizer: SymmetryVisualizer,
}

impl SubVisualizer for BiSymmetryVisualizer {
    fn new() -> BiSymmetryVisualizer {
        BiSymmetryVisualizer {
            symmetry_visualizer: SymmetryVisualizer::new(),
        }
    }

    fn post_setup(&mut self, program_id: u32, framebuffer_id: u32) {
        self.symmetry_visualizer.post_setup(program_id, framebuffer_id);
    }

    fn update(&mut self, audio_frame: audio::AudioFrame) {
        self.symmetry_visualizer.update(audio_frame);
    }

    fn render_to_texture(&self, gl: &gfx::gl::Gl) {
        self.symmetry_visualizer.render_to_texture(gl);
    }

    fn vs_src(&self) -> &[u8] {
        b"
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

void main() {
    v_position = position;
    gl_Position = vec4(v_position, 0.0, 1.0);
}
\0"
    }

    fn fs_src(&self) -> &[u8] {
        b"
#version 100
precision mediump float;

#define PI 3.1415926535897932384626433832795

uniform float phase;

// Interpolated from the Vertex Shader.
varying vec2 v_position;

mat2 rotation(float theta) {
    float c = cos(theta);
    float s = sin(theta);
    return mat2(
        vec2(c, -s),
        vec2(s, c)
    );
}

bool in_wave(vec2 p, vec2 wave) {
    float x_epsilon = 0.025;
    float y_epsilon = 0.025;
    return (
        p.x <= wave.x + x_epsilon && p.x >= wave.x - x_epsilon &&
        p.y <= wave.y + y_epsilon && p.y >= wave.y - y_epsilon);
}

vec2 wave_position(float phase, vec2 scale, vec2 translate) {
    vec2 p = vec2(
        (((phase / PI) - 1.0) + translate.x) * scale.x,
        (sin(phase) + translate.y) * scale.y
    );
    // p *= rotation(PI / 4.0);
    return p;
}

void main() {
    if ((v_position.x * v_position.x) + (v_position.y * v_position.y) > 1.0) {
        // Out of bounds.
        gl_FragColor = vec4(0.0);
    } else {
        vec2 wave = wave_position(phase, vec2(0.8), vec2(0.0));

        vec2 p1 = v_position;
        vec2 p2 = vec2(-v_position.x, v_position.y);
        if (in_wave(p1, wave) || in_wave(p2, wave)) {
            gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        } else {
            gl_FragColor = vec4(0.0);
        }
    }
}
\0"
    }
}
