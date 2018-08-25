use audio;
use gfx;
use visualizer::symmetry::symmetry::SymmetryVisualizer;
use visualizer::visualizer::SubVisualizer;

pub struct QuadSymmetryVisualizer {
    symmetry_visualizer: SymmetryVisualizer,
}

impl SubVisualizer for QuadSymmetryVisualizer {
    fn new() -> QuadSymmetryVisualizer {
        QuadSymmetryVisualizer {
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
#define TAU PI * 2.0

uniform float phase;
uniform float speed;

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

float hue2rgb(float f1, float f2, float hue) {
    if (hue < 0.0) {
        hue += 1.0;
    } else if (hue > 1.0) {
        hue -= 1.0;
    }

    float res;
    if ((6.0 * hue) < 1.0) {
        res = f1 + (f2 - f1) * 6.0 * hue;
    } else if ((2.0 * hue) < 1.0) {
        res = f2;
    } else if ((3.0 * hue) < 2.0) {
        res = f1 + (f2 - f1) * ((2.0 / 3.0) - hue) * 6.0;
    } else {
        res = f1;
    }
    return res;
}

vec3 hsl2rgb(vec3 hsl) {
    vec3 rgb;

    if (hsl.y == 0.0) {
        rgb = vec3(hsl.z);
    } else {
        float f2;

        if (hsl.z < 0.5) {
            f2 = hsl.z * (1.0 + hsl.y);
        } else {
            f2 = hsl.z + hsl.y - hsl.y * hsl.z;
        }

        float f1 = 2.0 * hsl.z - f2;

        rgb.r = hue2rgb(f1, f2, hsl.x + (1.0/3.0));
        rgb.g = hue2rgb(f1, f2, hsl.x);
        rgb.b = hue2rgb(f1, f2, hsl.x - (1.0/3.0));
    }
    return rgb;
}

vec3 hsl2rgb(float h, float s, float l) {
    return hsl2rgb(vec3(h, s, l));
}

vec2 y_sin_wave(float phase, mat2 rotation_matrix, vec2 scale, vec2 translate) {
    vec2 p = vec2(
        fract(phase / TAU) * 2.0 - 1.0,
        sin(phase)
    );
    p *= scale;
    p += translate;
    p *= rotation_matrix;
    return p;
}

bool in_wave(vec2 p, vec2 wave, float x_epsilon, float y_epsilon) {
    return (
        p.x <= wave.x + x_epsilon && p.x >= wave.x - x_epsilon &&
        p.y <= wave.y + y_epsilon && p.y >= wave.y - y_epsilon);
}

bool all_in_wave(vec2 p1, vec2 p2, vec2 p3, vec2 p4, vec2 wave, float x_epsilon, float y_epsilon) {
    return (
        in_wave(p1, wave, x_epsilon, y_epsilon) ||
        in_wave(p2, wave, x_epsilon, y_epsilon) ||
        in_wave(p3, wave, x_epsilon, y_epsilon) ||
        in_wave(p4, wave, x_epsilon, y_epsilon)
    );

}

void main() {
    if ((v_position.x * v_position.x) + (v_position.y * v_position.y) > 1.0) {
        // Out of bounds.
        gl_FragColor = vec4(0.0);
    } else {
        vec2 p = v_position;
        p *= rotation(phase);
        vec2 p1 = p;
        vec2 p2 = p * rotation(TAU / 4.0);
        vec2 p3 = p * rotation(TAU * 2.0 / 4.0);
        vec2 p4 = p * rotation(TAU * 3.0 / 4.0);

        float phase_offset = 0.5;
        float min_phase = phase - phase_offset;
        float max_phase = phase + phase_offset;
        float phase_increment = phase_offset / 10.0;

        for (float t = min_phase; t <= max_phase; t += phase_increment) {
            vec2 wave = y_sin_wave(
                t, rotation(t),
                vec2(0.8 * fract(t / 10.0)), vec2(fract(t) - 0.5));

            if (all_in_wave(p1, p2, p3, p4, wave, 0.05, 0.05)) {
                gl_FragColor = vec4(hsl2rgb(fract(t * 0.05 / TAU), 1.0, speed * 1.2), 1.0);
                return;
            }

            wave = y_sin_wave(
                t, rotation(t),
                vec2(0.7 * fract(t / 9.0)), vec2(fract(t) - 0.5));

            if (all_in_wave(p1, p2, p3, p4, wave, 0.05, 0.05)) {
                gl_FragColor = vec4(hsl2rgb(fract((t + TAU / 3.0) * 0.05 / TAU), 1.0, speed * 1.2), 1.0);
                return;
            }

            wave = y_sin_wave(
                t, rotation(t),
                vec2(0.6 * fract(t / 8.0)), vec2(fract(t) - 0.5));

            if (all_in_wave(p1, p2, p3, p4, wave, 0.05, 0.05)) {
                gl_FragColor = vec4(hsl2rgb(fract((t + TAU * 2.0 / 3.0) * 0.05 / TAU), 1.0, speed * 1.2), 1.0);
                return;
            }
        }

        gl_FragColor = vec4(0.0);
    }
}
\0"
    }
}
