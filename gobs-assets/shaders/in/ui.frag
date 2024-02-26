#version 450

#extension GL_GOOGLE_include_directive: require

#include "ui.glsl"

layout(location = 0) in vec4 in_color;
layout(location = 1) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D diffuse_texture;
layout(set = 1, binding = 1) uniform sampler diffuse_sampler;

void main() {
	out_color = in_color * linear_to_gamma(texture(sampler2D(diffuse_texture, diffuse_sampler), in_uv));
}
