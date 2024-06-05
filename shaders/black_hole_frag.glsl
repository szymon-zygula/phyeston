#version 430

const float PI = 3.1415265;
const unsigned int INTEGRATION_STEPS = 500;
const unsigned int W1_STEPS = 500;
const float EPS = 0.00001;
const float NO_ROOT = -10000;

in vec3 world_position;

uniform vec3 eye_position;

out vec4 color;

uniform samplerCube skybox;

float calc_w1(float M, float b) {
    float w1 = 0.0;
    for(unsigned int i = 0; i < W1_STEPS; ++i) {
        w1 -= root(M, b, w1) / root_deriv(M, b, w1);

        if(abs(root(M, b, w1)) < eps) {
            return w1;
        }
    }

    return NO_ROOT;
}

float root(float M, float b, float w) {
    return 1 - w * w * (1 - 2 * M / b * w);
}

float root_deriv(float M, float b, float w) {
    return (-2 + 6 * M / b * w) * w;
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
        float sum += integrand(M, b, w) * dw;
    }
}

void main() {
    float w1 = calc_w1(M, b);
    if(w1 == NO_ROOT) {
        color = vec4(0, 0, 0, 0);
        return;
    }

    float dphi = delta_phi(M, b, w1);

    color = texture(skybox, vec3(-world_position.x, world_position.y, world_position.z));
}
