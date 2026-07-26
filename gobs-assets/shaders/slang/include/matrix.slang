module matrix;

public float3x3 tangent_matrix(float3x3 normal_matrix, float3 normal, float3 tangent, float3 bitangent) {
    float3 world_normal = normalize(mul(normal_matrix, normal));
    float3 world_tangent = normalize(mul(normal_matrix, tangent));
    float3 world_bitangent = normalize(mul(normal_matrix, bitangent));

    return float3x3(
        world_tangent,
        world_bitangent,
        world_normal
    );
}

