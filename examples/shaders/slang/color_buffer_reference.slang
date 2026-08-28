// alignment 16
struct Vertex {
    float3 position;
    float4 color;
}

struct PushConstants {
    Vertex* vertex_reference;
}
[[vk::push_constant]]
PushConstants push_constants;

struct VSOutput {
    float4 position: SV_POSITION;
    float4 color;
}

Vertex get_vertex(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    return  push_constants.vertex_reference[vertex_index];
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    Vertex v = get_vertex(vertex_index, instance_index);

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

