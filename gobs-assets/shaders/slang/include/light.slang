module light;

public float3 phong_reflection(float3 normal, float3 position, float3 light_dir,
        float3 light_color, float3 view_position, float3 ambient_color) {
    float3 view_dir = normalize(view_position - position);
    float3 half_dir = normalize(view_dir + light_dir);

    float diffuse_strength = max(dot(normal, light_dir), 0.0);
    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);

    return (diffuse_strength + specular_strength) * light_color + ambient_color;
}

public float3 phong_reflection_normal(float3 normal, float3 position, float3 light_dir,
        float3 light_color,  float3 view_position, float3 ambient_color) {
    float3 tangent_normal = normal * 2.0 - 1.0;

    return phong_reflection(tangent_normal, position, light_dir, light_color, view_position, ambient_color);
}

