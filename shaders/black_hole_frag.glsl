#version 430

in vec3 world_position;

uniform vec3 eye_position;

out vec4 color;

uniform samplerCube skybox;

void main() {
    color = texture(skybox, vec3(-world_position.x, world_position.y, world_position.z));
}
