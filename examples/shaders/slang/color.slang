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
    float4 color;
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    IObjectData o = get_instance(instance_index);

    Vertex v = o.get_vertex(vertex_index );
    float4x4 world_matrix = o.get_world_matrix();

    VSOutput output;

    output.position = mul(scene_data.view_proj, mul(world_matrix, float4(v.position, 1.0)));
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

