#version 450

layout(location = 0) in struct VertexOutput {
	vec4 color;
} vertex_out;

layout(location = 0) out vec4 out_color;

void main() {
	out_color = vertex_out.color;
}
