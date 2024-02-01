layout(set = 0, binding = 0) uniform SceneData {
	vec3 camera_position;
	mat4 view_proj;
    vec3 light_direction;
    vec4 light_color;
} scene_data;
