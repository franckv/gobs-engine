#version 450

#extension GL_GOOGLE_include_directive: require
#extension GL_EXT_buffer_reference: require

#include "scene_nolight.glsl"

layout(location = 0) out vec4 out_color;

struct Vertex {
	vec3 position;
	vec4 color;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
	Vertex vertices[];
};

layout(push_constant) uniform constants {
	mat4 world_matrix;
	VertexBuffer vertex_buffer_address;
} push_constants;

void main() {
	Vertex v = push_constants.vertex_buffer_address.vertices[gl_VertexIndex];
	
	gl_Position = scene_data.view_proj * push_constants.world_matrix * vec4(v.position, 1.f);

	out_color = vec4(0., 1., 0., 1.);
}
