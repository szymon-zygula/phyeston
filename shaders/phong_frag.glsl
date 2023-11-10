#version 430

uniform vec3 eye_position;

uniform vec3 light_position;
uniform vec3 light_color;
uniform vec3 ambient;

uniform vec4 material_color;
uniform float material_diffuse;
uniform float material_specular;
uniform float material_specular_exp;

in PointData {
    vec3 normal;
    vec3 position;
} point;

out vec4 color;

void main() {
    vec3 to_eye = normalize(eye_position - point.position);
    vec3 to_light = normalize(light_position - point.position);

    float diffuse =  material_diffuse * max(dot(point.normal, to_light), 0.0);
    vec3 reflected = normalize(reflect(-to_light, point.normal));
    float specular =
        material_specular * pow(max(dot(reflected, to_eye), 0.0), material_specular_exp);

    color = vec4(
        (ambient + diffuse + specular) * material_color.rgb * light_color,
        material_color.a
    );
}
