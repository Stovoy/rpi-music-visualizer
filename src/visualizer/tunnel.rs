use std::ptr;
use std::mem;

use audio;
use gfx;
use gfx::gl;

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

            self.framebuffer_id = framebuffer_id;
        }
    }

    pub fn render_to_texture(&self, gl: &gfx::gl::Gl) {
        unsafe {
            gl_try!(gl; gl.UseProgram(self.program_id));

            let vertex_data: [f32; 3 * 6] = [
                -1.0, -1.0, 0.0,
                1.0, -1.0, 0.25,
                -1.0, 1.0, 0.5,
                -1.0, 1.0, 0.5,
                1.0, -1.0, 0.75,
                1.0, 1.0, 1.0,
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
            let brightness_attrib = gl_try!(gl; gl.GetAttribLocation(
                self.program_id, b"brightness\0".as_ptr() as *const _));
            gl_try!(gl; gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                3 * mem::size_of::<f32>() as gl::types::GLsizei,
                ptr::null(),
            ));
            gl_try!(gl; gl.VertexAttribPointer(
                brightness_attrib as gl::types::GLuint, 1, gl::FLOAT, 0,
                3 * mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * mem::size_of::<f32>()) as *const () as *const _,
            ));
            gl_try!(gl; gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint));
            gl_try!(gl; gl.EnableVertexAttribArray(brightness_attrib as gl::types::GLuint));

            let time = gl_try!(gl; gl.GetUniformLocation(
                self.program_id, b"time\0".as_ptr() as *const _));
            let mid_power = gl_try!(gl; gl.GetUniformLocation(
                self.program_id, b"mid_power\0".as_ptr() as *const _));
            let low_power_counter = gl_try!(gl; gl.GetUniformLocation(
                self.program_id, b"low_power_counter\0".as_ptr() as *const _));

            gl_try!(gl; gl.Uniform1f(time, self.time));
            gl_try!(gl; gl.Uniform1f(mid_power, self.mid_power));
            gl_try!(gl; gl.Uniform1f(low_power_counter, self.low_power_counter));

            gl_try!(gl; gl.ClearColor(0.0, 0.0, 0.0, 1.0));
            gl_try!(gl; gl.Clear(gl::COLOR_BUFFER_BIT));

            gl_try!(gl; gl.BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_id));
            gl_try!(gl; gl.DrawBuffers(1, [gl::COLOR_ATTACHMENT0].as_ptr()));
            gl_try!(gl; gl.DrawArrays(gl::TRIANGLES, 0, 3 * 6));
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

        self.time += 0.01;

        // println!("{}", self.z_position);
    }
}

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

attribute vec2 position;
attribute float brightness;

// Variables for the Fragment Shader.
varying vec2 v_position;
varying float v_brightness;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_position = position;
    v_brightness = brightness;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

#define PI 3.1415926535897932384626433832795

// Interpolated from the Vertex Shader.
varying vec2 v_position;
varying float v_brightness;

const int MAX_MARCHING_STEPS = 255;
const float MIN_DIST = 0.0;
const float MAX_DIST = 100.0;
const float EPSILON = 0.0001;

uniform float time;

uniform float mid_power;

uniform float low_power_counter;

/**
 * Signed distance function for a sphere centered at the origin with radius 1.0;
 */
float sphereSDF(vec3 p, float r) {
    return length(p) - r;
}

/**
 * Signed distance function for a cube centered at the origin
 * with dimensions specified by size.
 */
float boxSDF(vec3 p, vec3 size) {
    vec3 d = abs(p) - (size / 2.0);

    // Assuming p is inside the cube, how far is it from the surface?
    // Result will be negative or zero.
    float insideDistance = min(max(d.x, max(d.y, d.z)), 0.0);

    // Assuming p is outside the cube, how far is it from the surface?
    // Result will be positive or zero.
    float outsideDistance = length(max(d, 0.0));

    return insideDistance + outsideDistance;
}

/**
 * Signed distance function for an XY aligned cylinder centered at the origin with
 * height h and radius r.
 */
float cylinderSDF(vec3 p, float h, float r) {
    // How far inside or outside the cylinder the point is, radially
    float inOutRadius = length(p.xy) - r;

    // How far inside or outside the cylinder is, axially aligned with the cylinder
    float inOutHeight = abs(p.z) - h/2.0;

    // Assuming p is inside the cylinder, how far is it from the surface?
    // Result will be negative or zero.
    float insideDistance = min(max(inOutRadius, inOutHeight), 0.0);

    // Assuming p is outside the cylinder, how far is it from the surface?
    // Result will be positive or zero.
    float outsideDistance = length(max(vec2(inOutRadius, inOutHeight), 0.0));

    return insideDistance + outsideDistance;
}

float intersectSDF(float distA, float distB) {
    return max(distA, distB);
}

float unionSDF(float distA, float distB) {
    return min(distA, distB);
}

float differenceSDF(float distA, float distB) {
    return max(distA, -distB);
}

mat3 rotateX(float theta) {
    float c = cos(theta);
    float s = sin(theta);
    return mat3(
        vec3(1, 0, 0),
        vec3(0, c, -s),
        vec3(0, s, c)
    );
}

mat3 rotateY(float theta) {
    float c = cos(theta);
    float s = sin(theta);
    return mat3(
        vec3(c, 0, s),
        vec3(0, 1, 0),
        vec3(-s, 0, c)
    );
}

mat3 rotateZ(float theta) {
    float c = cos(theta);
    float s = sin(theta);
    return mat3(
        vec3(c, -s, 0),
        vec3(s, c, 0),
        vec3(0, 0, 1)
    );
}

vec3 opTx(vec3 p, mat4 m) {
    return (m * vec4(p, 1.0)).xyz;
}

/**
 * Signed distance function describing the scene.
 *
 * Absolute value of the return value indicates the distance to the surface.
 * Sign indicates whether the point is inside or outside the surface,
 * negative indicating inside.
 */
float sceneSDF(vec3 p) {
    vec3 translation = vec3(1.6, 0.0, 0.0);
    translation *= rotateY(low_power_counter * 2.0 * PI);

    p = opTx(p, mat4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(translation, 1.0)
    ));

    float cylinderRadius = cos(time) * 0.15 + 0.5;
    float cylinder1 = cylinderSDF(p, 2.0, cylinderRadius);
    float cylinder2 = cylinderSDF(rotateX(radians(90.0)) * p, 2.0, cylinderRadius);
    float cylinder3 = cylinderSDF(rotateY(radians(90.0)) * p, 2.0, cylinderRadius);

    float cube = boxSDF(p, vec3(1.8, 1.8, 1.8));

    float sphere = sphereSDF(p, 1.5);

    float ballOffset = 0.4 + 2.0 * mid_power;
    float ballRadius = 0.3;
    float balls = sphereSDF(p - vec3(ballOffset, 0.0, 0.0), ballRadius);
    balls = unionSDF(balls, sphereSDF(p + vec3(ballOffset, 0.0, 0.0), ballRadius));
    balls = unionSDF(balls, sphereSDF(p - vec3(0.0, ballOffset, 0.0), ballRadius));
    balls = unionSDF(balls, sphereSDF(p + vec3(0.0, ballOffset, 0.0), ballRadius));
    balls = unionSDF(balls, sphereSDF(p - vec3(0.0, 0.0, ballOffset), ballRadius));
    balls = unionSDF(balls, sphereSDF(p + vec3(0.0, 0.0, ballOffset), ballRadius));

    float result = unionSDF(
        cylinder1, unionSDF(cylinder2, cylinder3));
    result = differenceSDF(
        intersectSDF(cube, sphere),
        result);
    result = unionSDF(balls, result);

    return balls;
}

/**
 * Return the shortest distance from the eyepoint to the scene surface along
 * the marching direction. If no part of the surface is found between start and end,
 * return end.
 *
 * eye: the eye point, acting as the origin of the ray
 * marchingDirection: the normalized direction to march in
 * start: the starting distance away from the eye
 * end: the max distance away from the ey to march before giving up
 */
float shortestDistanceToSurface(vec3 eye, vec3 marchingDirection, float start, float end) {
    float depth = start;
    for (int i = 0; i < MAX_MARCHING_STEPS; i++) {
        float dist = sceneSDF(eye + depth * marchingDirection);
        if (dist < EPSILON) {
			return depth;
        }
        depth += dist;
        if (depth >= end) {
            return end;
        }
    }
    return end;
}

/**
 * Return the normalized direction to march in from the eye point for a single pixel.
 *
 * fieldOfView: vertical field of view in degrees
 * size: resolution of the output image
 * fragCoord: the x,y coordinate of the pixel in the output image
 */
vec3 rayDirection(float fieldOfView, vec2 size, vec2 fragCoord) {
    vec2 xy = fragCoord - size / 2.0;
    float z = size.y / tan(radians(fieldOfView) / 2.0);
    return normalize(vec3(xy, -z));
}

/**
 * Using the gradient of the SDF, estimate the normal on the surface at point p.
 */
vec3 estimateNormal(vec3 p) {
    return normalize(vec3(
        sceneSDF(vec3(p.x + EPSILON, p.y, p.z)) - sceneSDF(vec3(p.x - EPSILON, p.y, p.z)),
        sceneSDF(vec3(p.x, p.y + EPSILON, p.z)) - sceneSDF(vec3(p.x, p.y - EPSILON, p.z)),
        sceneSDF(vec3(p.x, p.y, p.z  + EPSILON)) - sceneSDF(vec3(p.x, p.y, p.z - EPSILON))
    ));
}

/**
 * Lighting contribution of a single point light source via Phong illumination.
 *
 * The vec3 returned is the RGB color of the light's contribution.
 *
 * ambient_color: Ambient color
 * diffuse_color: Diffuse color
 * specular_color: Specular color
 * alpha: Shininess coefficient
 * p: position of point being lit
 * eye: the position of the camera
 * lightPos: the position of the light
 * lightIntensity: color/intensity of the light
 *
 * See https://en.wikipedia.org/wiki/Phong_reflection_model#Description
 */
vec3 phongContribForLight(vec3 diffuse_color, vec3 specular_color, float alpha, vec3 p, vec3 eye,
                          vec3 lightPos, vec3 lightIntensity) {
    vec3 N = estimateNormal(p);
    vec3 L = normalize(lightPos - p);
    vec3 V = normalize(eye - p);
    vec3 R = normalize(reflect(-L, N));

    float dotLN = dot(L, N);
    float dotRV = dot(R, V);

    if (dotLN < 0.0) {
        // Light not visible from this point on the surface
        return vec3(0.0, 0.0, 0.0);
    }

    if (dotRV < 0.0) {
        // Light reflection in opposite direction as viewer, apply only diffuse
        // component
        return lightIntensity * (diffuse_color * dotLN);
    }
    return lightIntensity * (diffuse_color * dotLN + specular_color * pow(dotRV, alpha));
}

/**
 * Lighting via Phong illumination.
 *
 * The vec3 returned is the RGB color of that point after lighting is applied.
 * ambient_color: Ambient color
 * diffuse_color: Diffuse color
 * specular_color: Specular color
 * alpha: Shininess coefficient
 * p: position of point being lit
 * eye: the position of the camera
 *
 * See https://en.wikipedia.org/wiki/Phong_reflection_model#Description
 */
vec3 phongIllumination(vec3 ambient_color, vec3 diffuse_color, vec3 specular_color, float alpha, vec3 p, vec3 eye) {
    const vec3 ambientLight = 0.5 * vec3(1.0, 1.0, 1.0);
    vec3 color = ambientLight * ambient_color;

    vec3 light_intensity = vec3(0.4, 0.4, 0.4);
    float light_distance = 16.0;

    color += phongContribForLight(
        diffuse_color, specular_color, alpha, p, eye,
        vec3(light_distance, 0.0, 0.0), light_intensity);

    color += phongContribForLight(
        diffuse_color, specular_color, alpha, p, eye,
        vec3(-light_distance, 0.0, 0.0), light_intensity);

    color += phongContribForLight(
        diffuse_color, specular_color, alpha, p, eye,
        vec3(0.0, light_distance, 0.0), light_intensity);

    color += phongContribForLight(
        diffuse_color, specular_color, alpha, p, eye,
        vec3(0.0, -light_distance, 0.0), light_intensity);

    color += phongContribForLight(
        diffuse_color, specular_color, alpha, p, eye,
        vec3(0.0, 0.0, light_distance), light_intensity);

    color += phongContribForLight(
        diffuse_color, specular_color, alpha, p, eye,
        vec3(0.0, 0.0, -light_distance), light_intensity);

    return color;
}

/**
 * Return a transform matrix that will transform a ray from view space
 * to world coordinates, given the eye point, the camera target, and an up vector.
 *
 * This assumes that the center of the camera is aligned with the negative z axis in
 * view space when calculating the ray marching direction. See rayDirection.
 */
mat4 viewMatrix(vec3 eye, vec3 center, vec3 up) {
    // Based on gluLookAt man page
    vec3 f = normalize(center - eye);
    vec3 s = normalize(cross(f, up));
    vec3 u = cross(s, f);
    return mat4(
        vec4(s, 0.0),
        vec4(u, 0.0),
        vec4(-f, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}

void main() {
	vec3 viewDir = rayDirection(45.0, vec2(1024.0, 1024.0), gl_FragCoord.xy);

    float min_dist = 3.0;
    vec3 eye = vec3(
        5.0 + min_dist,
        5.0 + min_dist,
        5.0 + min_dist);

    mat4 viewToWorld = viewMatrix(eye, vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));

    vec3 worldDir = (viewToWorld * vec4(viewDir, 0.0)).xyz;

    float dist = shortestDistanceToSurface(eye, worldDir, MIN_DIST, MAX_DIST);

    if (dist > MAX_DIST - EPSILON) {
        // Didn't hit anything
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        // The closest point on the surface to the eyepoint along the view ray
        vec3 p = eye + dist * worldDir;

        vec3 diffuse_color = vec3(1.0, 0.0, 0.0);
        diffuse_color *= rotateY(low_power_counter * 2.0 * PI);

        vec3 ambient_color = vec3(0.2, 0.2, 0.2);
        vec3 specular_color = vec3(1.0, 1.0, 1.0);
        float shininess = 10.0;

        vec3 color = phongIllumination(
            ambient_color,
            diffuse_color,
            specular_color,
            shininess, p, eye);

        gl_FragColor = vec4(color, 1.0);
    }
}
\0";
