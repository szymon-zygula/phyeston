#version 430

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;

uniform mat4 model_transform;
uniform mat4 view_transform;
uniform mat4 projection_transform;

uniform vec3 eye_position;
out vec3 world_position;

void main() {
    vec4 world = model_transform * vec4(position, 1.0f);
    gl_Position = projection_transform * view_transform * world;
    world_position = world.xyz + eye_position *0.00000001;
}
