import common;
import display;
import vertex_layout;

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
    let diffuse_color = diffuse_texture.Sample(diffuse_sampler, input.uv);

    return input.color * diffuse_color;
}

