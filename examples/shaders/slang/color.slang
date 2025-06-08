struct SceneData {
    float3 camera_position;
	float4x4 view_proj;
	float3 light_direction;
	float4 light_color;
	float4 ambient_color;
}
[[vk::binding(0, 0)]]
ConstantBuffer<SceneData> scene_data;

// alignment 16
struct Vertex {
    float3 position;
    float pad;
    float4 color;
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
    output.color = v.color;

    return output;
}

[shader("fragment")]
float4 fragment_main(VSOutput input) {
    float4 color = input.color;

    return color;
}

static float near = 0.1;
static float far = 10.;

[shader("fragment")]
float4 depth_fragment_main(VSOutput input) {
    float depth = input.position.z; // 0..1
    float ndc = depth * 2. - 1.; // -1..1
    float linear_depth = ((2. * near * far) / (far + near - ndc * (far - near))); // near..far

	float4 color = float4(float3(linear_depth / far), 1.);

    return color;
}

