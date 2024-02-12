#version 450

#extension GL_EXT_buffer_reference: require

layout(set = 0, binding = 0) uniform SceneData {
	vec3 camera_position;
	mat4 view_proj;
} scene_data;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec2 out_uv;

struct Vertex {
	vec3 position;
	vec4 color;
	vec2 uv;
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
	
	vec4 world_position = push_constants.world_matrix * vec4(v.position, 1.f);

	gl_Position = scene_data.view_proj * world_position;
	out_color = v.color;
	out_uv = v.uv;
}
