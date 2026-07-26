// alignment 16
struct Vertex {
    float3 position;
    float4 color;
}

struct VSOutput {
    float4 position: SV_POSITION;
    float4 color;
}

[shader("vertex")]
VSOutput vertex_main(Vertex v) {
    VSOutput output;
    output.position = float4(v.position.x, -v.position.y, v.position.z, 1.0);
    output.color = v.color;

    return output;
}

[shader("fragment")]
float4 fragment_main(VSOutput input) {
    float4 color = input.color;

    return color;
}

