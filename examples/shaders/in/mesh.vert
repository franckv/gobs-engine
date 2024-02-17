#version 450

#extension GL_GOOGLE_include_directive: require
#extension GL_EXT_buffer_reference: require

#include "light.glsl"
#include "scene_light.glsl"

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec3 out_tangent_position;
layout(location = 3) out vec3 out_tangent_view_position;
layout(location = 4) out vec3 out_tangent_light_dir;

struct Vertex {
	vec3 position;
	vec2 uv;
	vec3 normal;
	vec3 tangent;
	vec3 bitangent;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
	Vertex vertices[];
};

layout(push_constant) uniform constants {
	mat4 world_matrix;
	mat3 normal_matrix;
	VertexBuffer vertex_buffer_address;
} push_constants;

void main() {
	Vertex v = push_constants.vertex_buffer_address.vertices[gl_VertexIndex];
	
	mat3 tangent_matrix = tangent_matrix(push_constants.normal_matrix, v.normal, v.tangent, v.bitangent);

	vec4 world_position = push_constants.world_matrix * vec4(v.position, 1.f);

	gl_Position = scene_data.view_proj * world_position;
	out_uv = v.uv;
	out_normal = push_constants.normal_matrix * v.normal;

	out_tangent_position = tangent_matrix * world_position.xyz;
	out_tangent_view_position = tangent_matrix * scene_data.camera_position.xyz;
	out_tangent_light_dir = normalize(tangent_matrix * scene_data.light_direction);
}
