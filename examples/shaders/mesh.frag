#version 450

//shader input
layout(location = 0) in vec3 inColor;
layout(location = 1) in vec2 inUV;

//output write
layout(location = 0) out vec4 outFragColor;

//layout(set = 1, binding = 0) uniform sampler2D displayTexture;
layout(set = 1, binding = 0) uniform texture2D u_texture;
layout(set = 1, binding = 1) uniform sampler u_sampler;

void main()
{
	outFragColor = texture(sampler2D(u_texture, u_sampler), inUV) * vec4(inColor,1.0f);
}
