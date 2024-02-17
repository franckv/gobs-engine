#version 450

#extension GL_GOOGLE_include_directive: require

#include "light.glsl"
#include "scene_light.glsl"

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_tangent_position;
layout(location = 3) in vec3 in_tangent_view_position;
layout(location = 4) in vec3 in_tangent_light_dir;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D diffuse_texture;
layout(set = 1, binding = 1) uniform sampler diffuse_sampler;

void main() {
	vec4 object_color = texture(sampler2D(diffuse_texture, diffuse_sampler), in_uv);

	vec3 light = phong_reflection(in_normal, in_tangent_position, in_tangent_light_dir, scene_data.light_color.xyz, in_tangent_view_position, scene_data.ambient_color.xyz);

	out_color = vec4(light * object_color.xyz, object_color.a);
}
