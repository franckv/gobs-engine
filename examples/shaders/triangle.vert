#version 450
#extension GL_EXT_buffer_reference : require

layout (location = 0) out vec3 outColor;

struct Vertex {
	vec3 position;
	float uv_x;
	vec3 normal;
	float uv_y;
	vec4 color;
}; 

layout(buffer_reference, std430) readonly buffer VertexBuffer{ 
	Vertex vertices[];
};

//push constants block
layout( push_constant ) uniform constants
{	
	mat4 render_matrix;
	VertexBuffer vertexBuffer;
} PushConstants;

void main()
{
	//const array of positions for the triangle
	const vec3 positions[4] = vec3[4](
		vec3(0.5f,-0.5f, 0.0f),
		vec3(0.5f,0.5f, 0.0f),
		vec3(-0.5f,-0.5f, 0.0f),
		vec3(-0.5f,0.5f, 0.0f)
	);

	//const array of colors for the triangle
	const vec3 colors[4] = vec3[4](
		vec3(1.0f, 0.0f, 0.0f), //red
		vec3(0.0f, 1.0f, 0.0f), //green
		vec3(0.f, 0.0f, 1.0f),  //blue
		vec3(0.5f, 0.5f, 0.5f)  //grey
	);

	//output the position of each vertex
	gl_Position = PushConstants.render_matrix * vec4(positions[gl_VertexIndex], 1.0f);
	outColor = colors[gl_VertexIndex];
}
