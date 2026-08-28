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

interface IMaterialData {
    float4 get_diffuse_color();
    float4 sample_diffuse(float2 uv);
    float4 sample_normal(float2 uv);
}

struct Material: IMaterialData {
    float4 diffuse_color;
    uint diffuse_index;
    uint normal_index;

    float4 get_diffuse_color() {
        return diffuse_color;
    }

    float4 sample_diffuse(float2 uv) {
        return textures[NonUniformResourceIndex(diffuse_index)].Sample(sampler, uv);
    }

    float4 sample_normal(float2 uv) {
        return textures[NonUniformResourceIndex(normal_index)].Sample(sampler, uv);
    }
}
[[vk::binding(0, MATERIAL_DATA_SET)]]
ByteAddressBuffer materials;

IMaterialData get_material(uint material_offset) {
    return materials.Load<Material>(material_offset);
}

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

struct VSOutputColor {
    float4 position: SV_POSITION;
    float4 color;
    nointerpolation uint material_offset;
}

[shader("vertex")]
VSOutputColor vertex_color_main(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID) {
    IObjectData o = get_instance(instance_index);

    Vertex v = o.get_vertex(vertex_index);
    float4x4 world_matrix = o.get_world_matrix();
    uint material_offset = o.get_material_offset();

    VSOutputColor output;

    float4 world_position = mul(world_matrix, float4(v.position, 1.0));

    output.position = mul(scene_data.view_proj, world_position);
    output.color = v.color;
    output.material_offset = material_offset;

    return output;
}

[shader("fragment")]
float4 fragment_color_main(VSOutputColor input) {
    IMaterialData m = get_material(input.material_offset);

    return m.get_diffuse_color() * input.color;
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
    IMaterialData m = get_material(input.material_offset);

	float4 object_color = m.get_diffuse_color() * input.color;

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
    IMaterialData m = get_material(input.material_offset);

	float4 object_diffuse = m.sample_diffuse(input.uv);

	float4 object_color = m.get_diffuse_color() * object_diffuse;

	float3 light = phong_reflection(input.normal, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}

[shader("fragment")]
float4 fragment_texture_normal_main(VSOutputLight input) {
    IMaterialData m = get_material(input.material_offset);

	float4 object_diffuse = m.sample_diffuse(input.uv);
	float4 object_normal = m.sample_normal(input.uv);

	float4 object_color = m.get_diffuse_color() * object_diffuse;

	float3 light = phong_reflection_normal(object_normal.xyz, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}

