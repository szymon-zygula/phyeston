#version 430

out vec4 frag_color;

in vec2 tex;

uniform sampler2D texture_sampler;

void main() {
    frag_color = texture(texture_sampler, tex);
}
