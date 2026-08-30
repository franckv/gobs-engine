import common;
import object_data;

struct SceneData {
	float4x4 view_proj;
}
[[vk::binding(0, SCENE_DATA_SET)]]
ConstantBuffer<SceneData> scene_data;

struct PushConstants: IObjectData {
    float4x4 world_matrix;
    Vertex* vertex_reference;

    Vertex get_vertex(uint vertex_index) {
        return vertex_reference[vertex_index];
    }

    float4x4 get_world_matrix() {
        return world_matrix;
    }

    uint get_material_offset() {
        return 0;
    }
}

[[vk::push_constant]]
PushConstants push_constants;

IObjectData get_instance(uint instance_index) {
    return push_constants;
}

struct VSOutput {
    float4 position: SV_POSITION;
    float2 uv;
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    IObjectData o = get_instance(instance_index);

    Vertex v = o.get_vertex(vertex_index );
    float4x4 world_matrix = o.get_world_matrix();

    VSOutput output;

    output.position = mul(scene_data.view_proj, mul(world_matrix, float4(v.position, 1.0)));
    output.uv = v.uv;

    return output;
}

[shader("fragment")]
float4 fragment_main(VSOutput input) {
    int checker = int(floor(input.uv.x * 128.0)) + int(floor(input.uv.y * 128.0));
    float value = (checker % 2 == 0) ? 0.0 : 1.0;

    return float4(value, value, value, 1.0);
}

