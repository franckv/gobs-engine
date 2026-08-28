import common;
import object_data;

struct SceneData {
    float3 camera_position;
	float4x4 view_proj;
	float3 light_direction;
	float4 light_color;
	float4 ambient_color;
}
[[vk::binding(0, SCENE_DATA_SET)]]
ConstantBuffer<SceneData> scene_data;

interface IMaterialData {
    float4 get_diffuse_color();
    float4 get_emissive_color();
}

struct Material: IMaterialData {
    float4 diffuse_color;
    float4 emissive_color;

    float4 get_diffuse_color() {
        return diffuse_color;
    }

    float4 get_emissive_color() {
        return emissive_color;
    }
}
[[vk::binding(0, MATERIAL_DATA_SET)]]
ConstantBuffer<Material> material;

IMaterialData get_material() {
    return material;
}

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
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    IObjectData o = get_instance(instance_index);

    Vertex v = o.get_vertex(vertex_index );
    float4x4 world_matrix = o.get_world_matrix();

    VSOutput output;
    output.position = mul(scene_data.view_proj, mul(world_matrix, float4(v.position, 1.0)));

    return output;
}

[shader("fragment")]
float4 fragment_main(VSOutput input) {
    IMaterialData m = get_material();

    float4 color = m.get_diffuse_color() + m.get_emissive_color();

    return color;
}

