#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 f_normal;
layout(location = 1) in vec2 f_tex_coord;

layout(binding = 1) uniform sampler2D tex_sampler;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(texture(tex_sampler, f_tex_coord).rgb, 1.0);
}
