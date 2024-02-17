#version 450

#extension GL_GOOGLE_include_directive: require

#include "scene_light.glsl"

layout(location = 0) in vec4 in_color;

layout(location = 0) out vec4 out_color;

float near = 0.1;
float far = 10.;

void main() {
    float depth = gl_FragCoord.z; // 0..1
    float ndc = depth * 2. - 1.; // -1..1
    float linear_depth = ((2. * near * far) / (far + near - ndc * (far - near))); // near..far

	out_color = vec4(vec3(linear_depth / far), 1.);
}
