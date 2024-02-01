#version 450

#extension GL_GOOGLE_include_directive: require
#extension GL_EXT_buffer_reference: require

#include "scene.glsl"

layout(location = 0) out vec3 out_color;
layout(location = 1) out vec2 out_uv;
layout(location = 2) out vec3 out_normal;

struct Vertex {
	vec3 position;
	vec4 color;
	vec2 uv;
	vec3 normal;
}; 

layout(buffer_reference, std430) readonly buffer VertexBuffer{ 
	Vertex vertices[];
};

//push constants block
layout(push_constant) uniform constants
{	
	mat4 world_matrix;
	VertexBuffer vertex_buffer_address;
} push_constants;

void main() 
{	
	//load vertex data from device adress
	Vertex v = push_constants.vertex_buffer_address.vertices[gl_VertexIndex];

	//output data
	gl_Position = scene_data.view_proj * push_constants.world_matrix * vec4(v.position, 1.f);
	out_color = v.color.xyz;
	out_uv = v.uv;
	out_normal = (push_constants.world_matrix * vec4(v.normal, 0.f)).xyz;
}
