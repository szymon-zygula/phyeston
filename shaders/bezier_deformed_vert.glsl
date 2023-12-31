#version 430

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

// Bezier cube
uniform vec3 bezier_cube[64];

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;

out PointData {
    vec3 normal;
    vec3 position;
} point;

vec3 bc(uint u, uint v, uint w) {
    return bezier_cube[w + v * 4 + u * 16];
}

vec3 bezier3(vec3 b0, vec3 b1, vec3 b2, vec3 b3, float t) {
    float t1 = 1.0f - t;

    b0 = t1 * b0 + t * b1;
    b1 = t1 * b1 + t * b2;
    b2 = t1 * b2 + t * b3;

    b0 = t1 * b0 + t * b1;
    b1 = t1 * b1 + t * b2;

    return t1 * b0 + t * b1;
}

vec3 bicubic_bezier(float u, float v, uint w) {
    vec3 p0 = bezier3(bc(0, 0, w), bc(0, 1, w), bc(0, 2, w), bc(0, 3, w), v);
    vec3 p1 = bezier3(bc(1, 0, w), bc(1, 1, w), bc(1, 2, w), bc(1, 3, w), v);
    vec3 p2 = bezier3(bc(2, 0, w), bc(2, 1, w), bc(2, 2, w), bc(2, 3, w), v);
    vec3 p3 = bezier3(bc(3, 0, w), bc(3, 1, w), bc(3, 2, w), bc(3, 3, w), v);

    return bezier3(p0, p1, p2, p3, u);
}

vec3 tricubic_bezier(float u, float v, float w) {
    vec3 p0 = bicubic_bezier(u, v, 0);
    vec3 p1 = bicubic_bezier(u, v, 1);
    vec3 p2 = bicubic_bezier(u, v, 2);
    vec3 p3 = bicubic_bezier(u, v, 3);

    return bezier3(p0, p1, p2, p3, w);
}

const float SMALL_SCALE = 1e-4;

void main() {
    float u = position.x;
    float v = position.y;
    float w = position.z;

    vec3 org_world = (model * vec4(position, 1.0)).xyz;

    vec3 deformed = tricubic_bezier(org_world.x, org_world.y, org_world.z);

    vec3 org_world_small = org_world - normal * SMALL_SCALE;
    vec3 deformed_small = tricubic_bezier(org_world_small.x, org_world_small.y, org_world_small.z);
    vec3 deformed_normal = normalize(deformed - deformed_small);

    gl_Position = projection * view * vec4(deformed, 1.0f);
    point.position = deformed;
    point.normal = normalize((model * vec4(deformed_normal, 0.0f)).xyz);
}
