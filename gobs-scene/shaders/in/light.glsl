
mat3 tangent_matrix(mat3 normal_matrix, vec3 normal, vec3 tangent, vec3 bitangent) {
    vec3 world_normal = normalize(normal_matrix * normal);
    vec3 world_tangent = normalize(normal_matrix * tangent);
    vec3 world_bitangent = normalize(normal_matrix * bitangent);
    return transpose(mat3(
        world_tangent,
        world_bitangent,
        world_normal
    ));
}

vec3 light_simple(vec3 normal, vec3 light_dir, vec3 light_color, vec3 ambient_color) {
	float light = max(dot(normal, light_dir), 0.0f);

	return light * light_color + ambient_color;
}

vec3 phong_reflection(vec3 normal, vec3 position, vec3 light_dir, vec3 light_color,  vec3 view_position, vec3 ambient_color) {
    vec3 view_dir = normalize(view_position - position);
    vec3 half_dir = normalize(view_dir + light_dir);

    float diffuse_strength = max(dot(normal, light_dir), 0.0);
    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);

    return (diffuse_strength + specular_strength) * light_color + ambient_color;
}

vec3 phong_reflection_normal(vec3 normal, vec3 position, vec3 light_dir, vec3 light_color,  vec3 view_position, vec3 ambient_color) {
    vec3 tangent_normal = normal * 2.0 - 1.0;
    
    return phong_reflection(tangent_normal, position, light_dir, light_color, view_position, ambient_color);
}
