#version 430

const float PI = 3.1415265;
const unsigned int INTEGRATION_STEPS = 500;
const unsigned int W1_STEPS = 4;
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
    vec3 eye_black_point = dot(eye_vec_norm, -eye_position) * eye_vec_norm;
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
    float prev = 100.0;
    for(unsigned int i = 0; i < W1_STEPS; ++i) {
        prev = w1;
        w1 -= root(M, b, w1) / root_deriv(M, b, w1);

        if(abs(prev - w1) < EPS) {
            return w1;
        }
    }

    return NO_ROOT;
}

float integrand(float M, float b, float w) {
    return pow(root(M, b, w), -0.5);
}

float delta_phi(float M, float b, float w1) {
    float sum = 0.0;
    float dw = w1 / INTEGRATION_STEPS;
    float w = dw / 2;

    for(unsigned int i = 0; i < INTEGRATION_STEPS; ++i) {
        w += dw;
        sum += integrand(M, b, w) * dw;
    }

    return 2 * sum - PI;
}

mat3 rotate(float dphi) {
    return mat3( cos(dphi), 0, sin(dphi),
                 0,         1, 0,
                -sin(dphi), 0, cos(dphi));
}

vec3 deflect(float dphi) {
    // change of origin + change of basis + rotation + change of basis + change of length + change of origin
    vec3 from_eye = world_position - eye_position;

    vec3 to_hole = -normalize(eye_position);
    vec3 perpen = normalize(cross(from_eye, to_hole));
    vec3 perpen_2 = normalize(cross(to_hole, perpen));

    mat3 basis = mat3(
        to_hole.x, perpen.x, perpen_2.x,
        to_hole.y, perpen.y, perpen_2.y,
        to_hole.z, perpen.z, perpen_2.z
    );

    vec3 from_eye_new_basis = transpose(basis) * from_eye;
    vec3 from_eye_new_basis_deflected = rotate(dphi) * from_eye_new_basis;
    vec3 from_eye_deflected = basis * from_eye_new_basis_deflected;

    return from_eye_deflected + eye_position;
}

void main() {
    float b = calc_b();
    float w1 = calc_w1(M, b);
    if(w1 == NO_ROOT) {
        color = vec4(1, b, 0, 1);
        return;
    }

    float dphi = delta_phi(M, b, w1);
    vec3 dir = deflect(dphi);

    color = texture(skybox, dir);
}
