#version 450
#extension GL_EXT_buffer_reference : require

layout (location = 0) out vec3 outColor;

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

layout(set=0, binding=0) uniform SceneData {
	vec3 camera_position;
	mat4 view_proj;
} scene_data;

void main() 
{	
	//load vertex data from device adress
	Vertex v = push_constants.vertex_buffer_address.vertices[gl_VertexIndex];

	//output data
	gl_Position = scene_data.view_proj * push_constants.world_matrix * vec4(v.position, 1.0f);
	outColor = v.color.xyz;
}
