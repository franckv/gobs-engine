#version 450

//shader input
layout(location = 0) in vec3 inColor;
layout(location = 1) in vec2 inUV;

//output write
layout(location = 0) out vec4 outFragColor;

layout(set = 1, binding = 0) uniform sampler2D displayTexture;

void main()
{
	outFragColor = texture(displayTexture, inUV) * vec4(inColor,1.0f);
}
