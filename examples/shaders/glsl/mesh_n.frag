#version 450

#extension GL_GOOGLE_include_directive: require

#include "includes/light.glsl"

layout(set = 0, binding = 0) uniform SceneData {
	vec3 camera_position;
	mat4 view_proj;
	vec3 light_direction;
	vec4 light_color;
	vec4 ambient_color;
} scene_data;

layout(location = 0) in struct VertexOutput {
	vec2 uv;
	vec3 normal;
 	vec3 tangent_position;
 	vec3 tangent_view_position;
	vec3 tangent_light_dir;
} vertex_out;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D diffuse_texture;
layout(set = 1, binding = 1) uniform sampler diffuse_sampler;
layout(set = 1, binding = 2) uniform texture2D normal_texture;
layout(set = 1, binding = 3) uniform sampler normal_sampler;

void main() {
	vec4 object_color = texture(sampler2D(diffuse_texture, diffuse_sampler), vertex_out.uv);
	vec4 object_normal = texture(sampler2D(normal_texture, normal_sampler), vertex_out.uv);

	vec3 light = phong_reflection_normal(object_normal.xyz, vertex_out.tangent_position, vertex_out.tangent_light_dir, scene_data.light_color.xyz, vertex_out.tangent_view_position, scene_data.ambient_color.xyz);

	out_color = vec4(light * object_color.xyz, object_color.a);
}
