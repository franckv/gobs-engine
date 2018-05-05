#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_uv;

layout(location = 3) in mat4 transform;
layout(location = 7) in mat3 normal_transform;
layout(location = 10) in vec4 color;
layout(location = 11) in vec4 region;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_tex;
layout(location = 2) out vec4 v_region;
layout(location = 3) out vec4 v_world_pos;
layout(location = 4) out vec3 v_normal;
layout(location = 5) out vec4 v_light_pos;

layout(set = 0, binding = 0) uniform MatrixData {
    mat4 projection;
} matrix;

layout(set = 0, binding = 1) uniform LightData {
    vec4 light_dir;
    vec4 light_color;
    vec4 light_ambient;
} light;

void main() {
    v_color = color;
    v_tex = tex_uv;
    v_region = region;
    v_world_pos = transform * vec4(position, 1.0);
    v_normal = normal_transform * normal;
    v_light_pos = transform * light.light_dir;

    gl_Position = matrix.projection * v_world_pos;
}
