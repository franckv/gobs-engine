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

[[vk::binding(0, MATERIAL_TEXTURES_SET)]]
Texture2D diffuse_texture;
[[vk::binding(1, MATERIAL_TEXTURES_SET)]]
SamplerState diffuse_sampler;
[[vk::binding(2, MATERIAL_TEXTURES_SET)]]
Texture2D normal_texture;
[[vk::binding(3, MATERIAL_TEXTURES_SET)]]
SamplerState normal_sampler;

struct PushConstants {
    float4x4 world_matrix;
    ConstBufferPointer<Vertex> vertex_reference;
}
[[vk::push_constant]]
PushConstants push_constants;

struct VSOutput {
    float4 position: SV_POSITION;
    float2 uv;
    float3 normal;
    float3 tangent_position;
    float3 tangent_view_position;
    float3 tangent_light_dir;
}

[shader("vertex")]
VSOutput vertex_main(uint vertex_index: SV_VertexID) {
    Vertex v = push_constants.vertex_reference[vertex_index];
    VSOutput output;

    float4x4 world_matrix = push_constants.world_matrix;
    // TODO: only works for uniform scaling
    float3x3 normal_matrix = (float3x3)world_matrix;
    float3x3 tangent_matrix = tangent_matrix(normal_matrix, v.normal, v.tangent, v.bitangent); 

    float4 world_position = mul(world_matrix, float4(v.position, 1.0));

    output.position = mul(scene_data.view_proj, world_position);
    output.uv = v.uv;
    output.normal = mul(normal_matrix, v.normal);
    output.tangent_position = mul(tangent_matrix, world_position.xyz);
    output.tangent_view_position = mul(tangent_matrix, scene_data.camera_position.xyz);
    output.tangent_light_dir = normalize(mul(tangent_matrix, scene_data.light_direction));

    return output;
}

[shader("fragment")]
float4 fragment_main(VSOutput input) {
	float4 object_color = diffuse_texture.Sample(diffuse_sampler, input.uv);

	float3 light = phong_reflection(input.normal, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}

[shader("fragment")]
float4 fragment_normal_main(VSOutput input) {
	float4 object_color = diffuse_texture.Sample(diffuse_sampler, input.uv);
	float4 object_normal = normal_texture.Sample(normal_sampler, input.uv);

	float3 light = phong_reflection_normal(object_normal.xyz, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}

