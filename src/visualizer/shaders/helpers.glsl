#version 100
precision mediump float;

#define PI 3.1415926535897932384626433832795

// Interpolated from the Vertex Shader.
varying vec2 v_position;

uniform float time;

uniform float midPower;

uniform float lowPowerCounter;

const float EPSILON = 0.0001;

/**
 * Signed distance function for a sphere centered at the origin with radius r;
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
 * Lighting contribution of a single point light source via Phong illumination.
 *
 * The vec3 returned is the RGB color of the light's contribution.
 *
 * diffuseColor: Diffuse color
 * specularColor: Specular color
 * shininess: Shininess coefficient
 * p: position of point being lit
 * eye: the position of the camera
 * lightPosition: the position of the light
 * lightIntensity: color/intensity of the light
 * normal: normal vector from the point
 *
 * See https://en.wikipedia.org/wiki/Phong_reflection_model#Description
 */
vec3 addLight(vec3 diffuseColor, vec3 specularColor, float shininess, vec3 p, vec3 eye,
              vec3 lightPosition, vec3 lightIntensity, vec3 normal, float falloff) {
    float lightDistance = 1.0 / (length(lightPosition - p) * falloff);

    vec3 L = normalize(lightPosition - p);
    vec3 V = normalize(eye - p);
    vec3 R = normalize(reflect(-L, normal));

    float dotLN = dot(L, normal);
    float dotRV = dot(R, V);

    if (dotLN < 0.0) {
        // Light not visible from this point on the surface
        return vec3(0.0, 0.0, 0.0);
    }

    if (dotRV < 0.0) {
        // Light reflection in opposite direction as viewer, apply only diffuse
        // component
        return lightDistance * lightIntensity * (diffuseColor * dotLN);
    }
    return lightDistance * lightIntensity * (diffuseColor * dotLN + specularColor * pow(dotRV, shininess));
}
