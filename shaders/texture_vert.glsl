#version 430

layout (location = 0) in vec3 position;

out vec2 tex;

uniform mat4 model_transform;
uniform mat4 view_transform;

void main() {
    vec4 pos = model_transform * vec4(position, 1.0f);
    gl_Position = view_transform * pos;
    tex = (position.xy + vec2(0.25)) * 2.0;
}
