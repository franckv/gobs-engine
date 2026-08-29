import common;
import display;

public struct Vertex {
    public float3 position;
    public float4 color;
    public float2 uv;
}

struct SceneData {
	float2 screen_size;
}
[[vk::binding(0, SCENE_DATA_SET)]]
ConstantBuffer<SceneData> scene_data;

[[vk::binding(0, MATERIAL_TEXTURES_SET)]]
Texture2D diffuse_texture;
[[vk::binding(1, MATERIAL_TEXTURES_SET)]]
SamplerState diffuse_sampler;

struct PushConstants {
    Vertex* vertex_reference;
}
[[vk::push_constant]]
PushConstants push_constants;

struct VSOutput {
    float4 position: SV_POSITION;
    float4 color;
    float2 uv;
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID) {
    Vertex v = push_constants.vertex_reference[vertex_index];
    VSOutput output;

    output.position = screen_to_ndc(v.position, scene_data.screen_size);
    output.color = v.color;
    output.uv = v.uv;

    return output;
}

[shader("fragment")]
float4 fragment_main(VSOutput input) {
    // input.color is gamma corrected with egui
    // sampled textures are linear
    // Gamma correct diffuse color if swapchain is linear
    // Convert input color to linear if swapchain is srgb
    float4 diffuse_color = diffuse_texture.Sample(diffuse_sampler, input.uv);
    float3 input_color_corrected = gamma_to_linear(input.color).rgb;

    return float4(input_color_corrected * diffuse_color.rgb, input.color.a);
}

