#version 430

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;

out PointData {
    vec3 normal;
    vec3 position;
} point;

uniform mat4 model_transform;
uniform mat4 view_transform;
uniform mat4 projection_transform;

void main() {
    vec4 world = model_transform * vec4(position, 1.0f);
    gl_Position = projection_transform * view_transform * world;
    point.position = world.xyz;
    point.normal = normalize((model_transform * vec4(normal, 0.0)).xyz);
}
