#version 450

#extension GL_EXT_buffer_reference: require
#extension GL_GOOGLE_include_directive: require

#include "ui.glsl"

layout(set = 0, binding = 0) uniform SceneData {
	vec2 screen_size;
} scene_data;

layout(location = 0) out struct VertexOutput {
	vec4 color;
	vec2 uv;
} vertex_out;

struct Vertex {
	vec3 position;
	vec4 color;
	vec2 uv;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
	Vertex vertices[];
};

layout(push_constant) uniform constants {
	VertexBuffer vertex_buffer_address;
} push_constants;

void main() {
	Vertex v = push_constants.vertex_buffer_address.vertices[gl_VertexIndex];

	gl_Position = screen_to_ndc(v.position, scene_data.screen_size);
	vertex_out.color = v.color;
	vertex_out.uv = v.uv;
}
