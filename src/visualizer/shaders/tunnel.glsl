int NUM_SPHERES = 6;
float TUNNEL_RADIUS = 0.8;
float TUNNEL_LENGTH = 50.0;

vec2 getSpherePosition(int num) {
    float theta = 2.0 * PI / float(NUM_SPHERES) * float(num);
    float x = cos(theta) * TUNNEL_RADIUS;
    float y = sin(theta) * TUNNEL_RADIUS;

    return vec2(x, y);
}

float sceneSDF(vec3 p) {
    float cylinder = cylinderSDF(p, TUNNEL_LENGTH, TUNNEL_RADIUS);
    float innerCylinder = cylinderSDF(p, TUNNEL_LENGTH + 0.1, 0.79);

    float tunnel = differenceSDF(cylinder, innerCylinder);

    float result = tunnel;

    //for (int i = 0; i < NUM_SPHERES; i++) {
    //    float sphere = sphereSDF(
    //        p - getSpherePosition(i),
    //        0.05);
    //
    //    result = unionSDF(result, sphere);
    //}

    return result;
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

vec3 getColor(vec3 p, vec3 eye) {
    vec3 color = vec3(0.0);
    vec3 normal = estimateNormal(p);

    float shininess = 10.0;

    for (int i = -10; i < 10; i++) {
        // Tunnel light.
        vec3 diffuseColor = vec3(1.0, 0.0, 0.0);
        vec3 specularColor = vec3(1.0, 0.0, 0.0);

        color += addLight(
            diffuseColor,
            specularColor,
            shininess, p, eye,
            vec3(
                getSpherePosition(i) * vec2(0.9, 0.9),
                -25.0 + fract(time / 8.0) * 30.0 + float(i) * 5.0),
            vec3(1.0),
            normal,
            6.0);
    }

    // Overhead light.
    vec3 diffuseColor = vec3(1.0);
    vec3 specularColor = vec3(1.0);
    color += addLight(
        diffuseColor,
        specularColor,
        shininess, p, eye,
        vec3(0.0, TUNNEL_RADIUS - 0.3,
            -25.0 + fract(time / 8.0) * 50.0),
        vec3(1.0),
        normal,
        3.0);

    return color;
}

vec3 getEye() {
    // return vec3(8.0, 8.0, 8.0);
    return vec3(0.0, 0.0, 25.0);
}
