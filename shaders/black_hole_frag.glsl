#version 430

const float PI = 3.1415265;

in vec3 world_position;

uniform vec3 eye_position;

out vec4 color;

uniform samplerCube skybox;

const unsigned int INTEGRATION_STEPS = 500;

float calc_w1(float M, float b) {

}

float integrand(float M, float b, w) {
    return 1.0 / sqrt(1 - w * w * (1 - 2 * M / b * w));
}

float delta_phi(float M, float b, float w1) {
    float sum = 0.0;
    float w1 = calc_w1(M, b)
    float dw = w1 / INTEGRATION_STEPS;
    float w = 0;

    for(int i = 0; i < INTEGRATION_STEPS; ++i)
    {
        w += dw;
        float sum += integrand(M, b, w) * dw;
    }
}

void main() {
    color = texture(skybox, vec3(-world_position.x, world_position.y, world_position.z));
}
