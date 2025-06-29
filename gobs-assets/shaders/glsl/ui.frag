#version 450

#extension GL_GOOGLE_include_directive: require

#include "ui.glsl"

layout(location = 0) in struct VertexOutput {
	vec4 color;
	vec2 uv;
} vertex_out;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D diffuse_texture;
layout(set = 1, binding = 1) uniform sampler diffuse_sampler;

void main() {
	out_color = vec4(vertex_out.color.rgb * linear_to_gamma(texture(sampler2D(diffuse_texture, diffuse_sampler), vertex_out.uv)).rgb, vertex_out.color.a);
}
