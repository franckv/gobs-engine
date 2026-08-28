import common;
import light;
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

struct Material {
    uint diffuse_index;
    uint normal_index;
}
[[vk::binding(0, MATERIAL_DATA_SET)]]
ByteAddressBuffer materials;

[[vk::binding(0, MATERIAL_TEXTURES_SET)]]
SamplerState sampler;
[[vk::binding(1, MATERIAL_TEXTURES_SET)]]
Texture2D textures[256];

struct InstanceData: IObjectData {
    float4x4 world_matrix;
    Vertex* vertex_reference;
    uint material_offset;

    Vertex get_vertex(uint vertex_index) {
        return vertex_reference[vertex_index];
    }

    float4x4 get_world_matrix() {
        return world_matrix;
    }

    uint get_material_offset() {
        return material_offset;
    }
}

struct PushConstants {
    InstanceData* instance_reference;
}
[[vk::push_constant]]
PushConstants push_constants;

IObjectData get_instance(uint instance_index) {
    return push_constants.instance_reference[instance_index];
}

struct VSOutput {
    float4 position: SV_POSITION;
    float2 uv;
    float3 normal;
    float3 tangent_position;
    float3 tangent_view_position;
    float3 tangent_light_dir;
    nointerpolation uint material_offset;
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    IObjectData o = get_instance(instance_index);

    Vertex v = o.get_vertex(vertex_index );
    float4x4 world_matrix = o.get_world_matrix();
    uint material_offset = o.get_material_offset();

    // TODO: only works for uniform scaling
    float3x3 normal_matrix = (float3x3)world_matrix;
    float3x3 tangent_matrix = tangent_matrix(normal_matrix, v.normal, v.tangent, v.bitangent); 
    float4 world_position = mul(world_matrix, float4(v.position, 1.0));

    VSOutput output;

    output.position = mul(scene_data.view_proj, world_position);
    output.uv = v.uv;
    output.normal = mul(tangent_matrix, mul(normal_matrix, v.normal));
    output.tangent_position = mul(tangent_matrix, world_position.xyz);
    output.tangent_view_position = mul(tangent_matrix, scene_data.camera_position.xyz);
    output.tangent_light_dir = normalize(mul(tangent_matrix, scene_data.light_direction));
    output.material_offset = material_offset;

    return output;
}

[shader("fragment")]
float4 fragment_normal_main(VSOutput input) {
    Material material = materials.Load<Material>(input.material_offset);

	float4 object_color = textures[NonUniformResourceIndex(material.diffuse_index)].Sample(sampler, input.uv);
	float4 object_normal = textures[NonUniformResourceIndex(material.normal_index)].Sample(sampler, input.uv);

	float3 light = phong_reflection_normal(object_normal.xyz, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}

