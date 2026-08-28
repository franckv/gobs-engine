import common;
import light;
import vertex_layout;

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
    float4 diffuse_color;
    uint diffuse_index;
    uint normal_index;
}
[[vk::binding(0, MATERIAL_DATA_SET)]]
ByteAddressBuffer materials;

[[vk::binding(0, MATERIAL_TEXTURES_SET)]]
SamplerState sampler;
[[vk::binding(1, MATERIAL_TEXTURES_SET)]]
Texture2D textures[256];

struct InstanceData {
    float4x4 world_matrix;
    Vertex* vertex_reference;
    uint material_offset;
}

struct PushConstants {
    InstanceData* instance_reference;
}
[[vk::push_constant]]
PushConstants push_constants;

struct VSOutputColor {
    float4 position: SV_POSITION;
    float4 color;
    nointerpolation uint material_offset;
}

[shader("vertex")]
VSOutputColor vertex_color_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    InstanceData instance = push_constants.instance_reference[instance_index];
    Vertex v = instance.vertex_reference[vertex_index];
    VSOutputColor output;

    float4x4 world_matrix = instance.world_matrix;

    float4 world_position = mul(world_matrix, float4(v.position, 1.0));

    output.position = mul(scene_data.view_proj, world_position);
    output.color = v.color;
    output.material_offset = instance.material_offset;

    return output;
}

[shader("fragment")]
float4 fragment_color_main(VSOutputColor input) {
    Material material = materials.Load<Material>(input.material_offset);

    return material.diffuse_color * input.color;
}

struct VSOutputLight {
    float4 position: SV_POSITION;
    float4 color;
    float2 uv;
    float3 normal;
    float3 tangent_position;
    float3 tangent_view_position;
    float3 tangent_light_dir;
    nointerpolation uint material_offset;
}

[shader("vertex")]
VSOutputLight vertex_color_light_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    InstanceData instance = push_constants.instance_reference[instance_index];
    Vertex v = instance.vertex_reference[vertex_index];
    VSOutputLight output;

    float4x4 world_matrix = instance.world_matrix;
    // TODO: only works for uniform scaling
    float3x3 normal_matrix = (float3x3)world_matrix;
    float3x3 tangent_matrix = tangent_matrix(normal_matrix, v.normal, v.tangent, v.bitangent); 

    float4 world_position = mul(world_matrix, float4(v.position, 1.0));

    output.position = mul(scene_data.view_proj, world_position);
    output.color = v.color;
    output.normal = mul(tangent_matrix, mul(normal_matrix, v.normal));
    output.tangent_position = mul(tangent_matrix, world_position.xyz);
    output.tangent_view_position = mul(tangent_matrix, scene_data.camera_position.xyz);
    output.tangent_light_dir = normalize(mul(tangent_matrix, scene_data.light_direction));
    output.material_offset = instance.material_offset;

    return output;
}

[shader("fragment")]
float4 fragment_color_light_main(VSOutputLight input) {
    Material material = materials.Load<Material>(input.material_offset);

	float4 object_color = material.diffuse_color * input.color;

	float3 light = phong_reflection(input.normal, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}

[shader("vertex")]
VSOutputLight vertex_texture_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    InstanceData instance = push_constants.instance_reference[instance_index];
    Vertex v = instance.vertex_reference[vertex_index];
    VSOutputLight output;

    float4x4 world_matrix = instance.world_matrix;
    // TODO: only works for uniform scaling
    float3x3 normal_matrix = (float3x3)world_matrix;
    float3x3 tangent_matrix = tangent_matrix(normal_matrix, v.normal, v.tangent, v.bitangent); 

    float4 world_position = mul(world_matrix, float4(v.position, 1.0));

    output.position = mul(scene_data.view_proj, world_position);
    output.uv = v.uv;
    output.normal = mul(tangent_matrix, mul(normal_matrix, v.normal));
    output.tangent_position = mul(tangent_matrix, world_position.xyz);
    output.tangent_view_position = mul(tangent_matrix, scene_data.camera_position.xyz);
    output.tangent_light_dir = normalize(mul(tangent_matrix, scene_data.light_direction));
    output.material_offset = instance.material_offset;

    return output;
}

[shader("fragment")]
float4 fragment_texture_main(VSOutputLight input) {
    Material material = materials.Load<Material>(input.material_offset);

	float4 object_diffuse = textures[NonUniformResourceIndex(material.diffuse_index)].Sample(sampler, input.uv);

	float4 object_color = material.diffuse_color * object_diffuse;

	float3 light = phong_reflection(input.normal, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}

[shader("fragment")]
float4 fragment_texture_normal_main(VSOutputLight input) {
    Material material = materials.Load<Material>(input.material_offset);

	float4 object_diffuse = textures[NonUniformResourceIndex(material.diffuse_index)].Sample(sampler, input.uv);
	float4 object_normal = textures[NonUniformResourceIndex(material.normal_index)].Sample(sampler, input.uv);

	float4 object_color = material.diffuse_color * object_diffuse;

	float3 light = phong_reflection_normal(object_normal.xyz, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}

