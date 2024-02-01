#version 450

#extension GL_GOOGLE_include_directive: require

#include "scene.glsl"

//shader input
layout(location = 0) in vec3 in_color;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec3 in_normal;

//output write
layout(location = 0) out vec4 out_color;

//layout(set = 1, binding = 0) uniform sampler2D displayTexture;
layout(set = 1, binding = 0) uniform texture2D u_texture;
layout(set = 1, binding = 1) uniform sampler u_sampler;

void main()
{
	float light = max(dot(in_normal, scene_data.light_direction), 0.1f);
	vec3 color = texture(sampler2D(u_texture, u_sampler), in_uv).xyz * in_color;
	out_color = vec4(
		(0.5 * light * scene_data.light_color.xyz + scene_data.ambient_color.xyz) * color,
		1.f);
}
