#version 450

#extension GL_EXT_buffer_reference: require

layout(set = 0, binding = 0) uniform SceneData {
	vec3 camera_position;
	mat4 view_proj;
    vec3 light_direction;
    vec4 light_color;
    vec4 ambient_color;
} scene_data;


layout(location = 0) out struct VertexOutput {
	vec4 color;
} vertex_out;

struct Vertex {
	vec3 position;
	vec4 color;
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

	gl_Position = scene_data.view_proj * push_constants.world_matrix * vec4(v.position, 1.f);

	vertex_out.color = v.color;
}
