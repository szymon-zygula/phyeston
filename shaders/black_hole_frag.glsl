#version 430

const float PI = 3.1415265;
const unsigned int INTEGRATION_STEPS = 500;
const unsigned int W1_STEPS = 500;
const float EPS = 0.00001;
const float NO_ROOT = -10000;

in vec3 world_position;

uniform vec3 eye_position;
uniform float M;

out vec4 color;

uniform samplerCube skybox;

float calc_b() {
    vec3 eye_vec = world_position - eye_position;
    vec3 eye_vec_norm = normalize(eye_vec);
    vec3 eye_black_point = dot(eye_vec_norm, -eye_position) * eye_vec;
    vec3 b_vec = eye_black_point + eye_position;
    return length(b_vec);
}

float root(float M, float b, float w) {
    return 1 - w * w * (1 - 2 * M / b * w);
}

float root_deriv(float M, float b, float w) {
    return (-2 + 6 * M / b * w) * w;
}

float calc_w1(float M, float b) {
    float w1 = 1.0;
    for(unsigned int i = 0; i < W1_STEPS; ++i) {
        w1 -= root(M, b, w1) / root_deriv(M, b, w1);

        if(abs(root(M, b, w1)) < EPS) {
            return w1;
        }
    }

    return NO_ROOT;
}

float integrand(float M, float b, float w) {
    return 1.0 / sqrt(root(M, b, w));
}

float delta_phi(float M, float b, float w1) {
    float sum = 0.0;
    float dw = w1 / INTEGRATION_STEPS;
    float w = 0;

    for(unsigned int i = 0; i < INTEGRATION_STEPS; ++i) {
        w += dw;
        sum += integrand(M, b, w) * dw;
    }

    return sum;
}

mat3 rotate(float dphi) {
    return mat3( cos(dphi), 0, sin(dphi),
                 0,         1, 0,
                -sin(dphi), 0, cos(dphi));
}

vec3 deflect(float dphi) {
    // Change of basis + rotation + change of basis
    vec3 from_eye = normalize(world_position - eye_position);
    vec3 to_hole = -normalize(eye_position);
    vec3 perpen = cross(from_eye, to_hole);
    vec3 perpen_2 = cross(to_hole, perpen);
    float from_eye_to_hole = dot(from_eye, to_hole);
    float from_eye_perpen_2 = dot(from_eye, perpen_2);
    vec3 new_from_eye = rotate(dphi) * vec3(from_eye_to_hole, 0, from_eye_perpen_2);
    vec3 new_from_eye_canon =
        new_from_eye.x * to_hole /*+ new_from_eye.y * perpen = 0*/ + new_from_eye.z * perpen_2;
    return new_from_eye_canon;
}

void main() {
    float b = calc_b();
    float w1 = calc_w1(M, b);
    if(w1 == NO_ROOT) {
        color = vec4(1, 0, 0, 1);
        return;
    }

    float dphi = delta_phi(M, b, w1);
    vec3 dir = deflect(dphi);

    color = texture(skybox, dir);
}
