#version 450

layout(set = 0, binding = 0) uniform SceneData {
	vec3 camera_position;
	mat4 view_proj;
    vec3 light_direction;
    vec4 light_color;
    vec4 ambient_color;
} scene_data;

layout(location = 0) in struct VertexOutput {
	vec4 color;
} vertex_out;

layout(location = 0) out vec4 out_color;

void main() {
	out_color = vertex_out.color;
}
