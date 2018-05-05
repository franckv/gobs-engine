#version 450

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_tex;
layout(location = 2) in vec4 v_region;
layout(location = 3) in vec4 v_world_pos;
layout(location = 4) in vec3 v_normal;
layout(location = 5) in vec4 v_light_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform MatrixData {
    mat4 projection;
} matrix;

layout(set = 0, binding = 1) uniform LightData {
    vec4 light_dir;
    vec4 light_color;
    vec4 light_ambient;
} light;

layout(set = 0, binding = 2) uniform sampler2D tex_sampler;

vec4 get_light(vec3 pos, vec3 normal_view, vec4 light_pos,
  vec3 light_color, vec3 light_ambient) {
  vec3 light_pos_surface;

  if (light_pos.w == 0.0) {
    light_pos_surface = normalize(light_pos.xyz);
  } else {
    light_pos_surface = normalize(light_pos.xyz - pos);
  }

  float light_diff = max(0, dot(normalize(light_pos_surface), normal_view));

  float attenuation = 1.0;

  return vec4(light_ambient + light_color * light_diff, 1.0);
}

vec4 get_texture_color(vec4 region, vec2 tex_uv) {
  vec2 offset = region.xy;
  vec2 scale = vec2(region.z - region.x, region.w - region.y);
  vec2 tex_coord = offset + scale * tex_uv;

  return texture(tex_sampler, tex_coord);
}

void main() {
    vec4 color = get_texture_color(v_region, v_tex) * v_color;
    vec4 light = get_light(v_world_pos.xyz, normalize(v_normal),
      v_light_pos, light.light_color.xyz, light.light_ambient.xyz);

    f_color = color * light;
}
