struct SceneData {
    float3 camera_position;
	float4x4 view_proj;
	float3 light_direction;
	float4 light_color;
	float4 ambient_color;
}
[[vk::binding(0, 0)]]
ConstantBuffer<SceneData> scene_data;

struct Material {
    float4 color;
}
[[vk::binding(1, 0)]]
ConstantBuffer<Material> material;

// alignment 16
struct Vertex {
    float3 position;
    float pad;
}

struct PushConstants {
    float4x4 world_matrix;
    float3x3 normal_matrix;
    Vertex* vertex_reference;
}
[[vk::push_constant]]
PushConstants push_constants;

struct VSOutput {
    float4 position: SV_POSITION;
    float4 color;
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID) {
    Vertex v = push_constants.vertex_reference[vertex_index];
    VSOutput output;
    output.position = mul(scene_data.view_proj, mul(push_constants.world_matrix, float4(v.position, 1.0)));
    output.color = material.color;

    return output;
}

[shader("fragment")]
float4 fragment_main(VSOutput input) {
    float4 color = input.color;

    return color;
}

