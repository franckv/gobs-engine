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

float near = 0.1;
float far = 10.;

void main() {
    float depth = gl_FragCoord.z; // 0..1
    float ndc = depth * 2. - 1.; // -1..1
    float linear_depth = ((2. * near * far) / (far + near - ndc * (far - near))); // near..far

	out_color = vec4(vec3(linear_depth / far), 1.);
}
