struct VSOutput {
    float4 position: SV_POSITION;
    float2 uv;
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID) {
    VSOutput output;

    float2 positions[3] = {
        float2(-0.5, -0.5),
        float2(0.5, -0.5),
        float2(0.0, 0.5),
    };
    float2 uvs[3] = {
        float2(0.0, 0.0),
        float2(1.0, 0.0),
        float2(0.5, 1.0),
    };

    output.position = float4(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];

    return output;
}

[[vk::binding(0, 2)]]
SamplerState tex_sampler;
[[vk::binding(1, 2)]]
Texture2D textures[256];

[shader("fragment")]
float4 fragment_main(VSOutput input) {
    return textures[0].Sample(tex_sampler, input.uv);
}
