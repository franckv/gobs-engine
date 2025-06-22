struct SceneData {
	float4x4 view_proj;
}
[[vk::binding(0, 0)]]
ConstantBuffer<SceneData> scene_data;

// alignment 16
struct Vertex {
    float3 position;
    float pad;
}

struct PushConstants {
    float4x4 world_matrix;
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
    output.color = float4(0., 1., 0., 1.);

    return output;
}

[shader("fragment")]
float4 fragment_main(VSOutput input) {
    float4 color = input.color;

    return color;
}

