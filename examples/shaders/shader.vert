#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_tex_coord;

layout(location = 3) in mat4 i_model;

layout(location = 0) out vec3 f_normal;
layout(location = 1) out vec2 f_tex_coord;

layout(binding = 0) uniform ViewProjection {
    mat4 matrix;
} u_view_proj;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = u_view_proj.matrix * (i_model * vec4(v_pos, 1.0));
    f_normal = v_normal;
    f_tex_coord = v_tex_coord;
}
