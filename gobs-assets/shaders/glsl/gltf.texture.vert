#version 450

#extension GL_GOOGLE_include_directive: require
#extension GL_EXT_buffer_reference: require

#include "light.glsl"

layout(set = 0, binding = 0) uniform SceneData {
	vec3 camera_position;
	mat4 view_proj;
    vec3 light_direction;
    vec4 light_color;
    vec4 ambient_color;
} scene_data;

layout(location = 0) out struct VertexOutput {
	vec2 uv;
	vec3 normal;
 	vec3 tangent_position;
 	vec3 tangent_view_position;
	vec3 tangent_light_dir;
} vertex_out;

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
	vertex_out.uv = v.uv;
	vertex_out.normal = push_constants.normal_matrix * v.normal;

	vertex_out.tangent_position = tangent_matrix * world_position.xyz;
	vertex_out.tangent_view_position = tangent_matrix * scene_data.camera_position.xyz;
	vertex_out.tangent_light_dir = normalize(tangent_matrix * scene_data.light_direction);
}
