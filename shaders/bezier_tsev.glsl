#version 430

layout (quads, equal_spacing, ccw) in;

uniform mat4 view;
uniform mat4 projection;
uniform uint invert_normals;

out PointData {
    vec3 normal;
    vec3 position;
} point;

// Control point
vec3 p(uint up, uint vp) {
    return gl_in[up * 4 + vp].gl_Position.xyz;
}

vec3 bezier2(vec3 b0, vec3 b1, vec3 b2, float t) {
    float t1 = 1.0f - t;

    b0 = t1 * b0 + t * b1;
    b1 = t1 * b1 + t * b2;

    return t1 * b0 + t * b1;
}

vec3 bezier3_diff(vec3 b0, vec3 b1, vec3 b2, vec3 b3, float t) {
    return bezier2(3.0 * (b1 - b0), 3.0 * (b2 - b1), 3.0 * (b3 - b2), t);
}

vec3 bezier3(vec3 b0, vec3 b1, vec3 b2, vec3 b3, float t) {
    float t1 = 1.0f - t;

    b0 = t1 * b0 + t * b1;
    b1 = t1 * b1 + t * b2;
    b2 = t1 * b2 + t * b3;

    return bezier2(b0, b1, b2, t);
}

vec3 bicubic_bezier(float u, float v) {
    vec3 p0 = bezier3(p(0, 0), p(0, 1), p(0, 2), p(0, 3), v);
    vec3 p1 = bezier3(p(1, 0), p(1, 1), p(1, 2), p(1, 3), v);
    vec3 p2 = bezier3(p(2, 0), p(2, 1), p(2, 2), p(2, 3), v);
    vec3 p3 = bezier3(p(3, 0), p(3, 1), p(3, 2), p(3, 3), v);

    return bezier3(p0, p1, p2, p3, u);
}

vec3 bicubic_bezier_norm(float u, float v) {
    vec3 v0 = bezier3_diff(p(0, 0), p(0, 1), p(0, 2), p(0, 3), v);
    vec3 v1 = bezier3_diff(p(1, 0), p(1, 1), p(1, 2), p(1, 3), v);
    vec3 v2 = bezier3_diff(p(2, 0), p(2, 1), p(2, 2), p(2, 3), v);
    vec3 v3 = bezier3_diff(p(3, 0), p(3, 1), p(3, 2), p(3, 3), v);
    vec3 dv = bezier3(v0, v1, v2, v3, u);

    vec3 u0 = bezier3_diff(p(0, 0), p(1, 0), p(2, 0), p(3, 0), u);
    vec3 u1 = bezier3_diff(p(0, 1), p(1, 1), p(2, 1), p(3, 1), u);
    vec3 u2 = bezier3_diff(p(0, 2), p(1, 2), p(2, 2), p(3, 2), u);
    vec3 u3 = bezier3_diff(p(0, 3), p(1, 3), p(2, 3), p(3, 3), u);
    vec3 du = bezier3(u0, u1, u2, u3, v);

    if(invert_normals == 1) {
        return normalize(cross(dv, du));
    }
    else {
        return normalize(cross(du, dv));
    }
}

void main() {
    float u = gl_TessCoord.x;
    float v = gl_TessCoord.y;

    point.normal = bicubic_bezier_norm(u, v);
    point.position = bicubic_bezier(u, v);
    gl_Position = projection * view * vec4(point.position, 1.0f);
}
