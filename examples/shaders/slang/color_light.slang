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

struct PushConstants {
    float4x4 world_matrix;
    Vertex* vertex_reference;
}
[[vk::push_constant]]
PushConstants push_constants;

struct VSOutputLight {
    float4 position: SV_POSITION;
    float4 color;
    float2 uv;
    float3 normal;
    float3 tangent_position;
    float3 tangent_view_position;
    float3 tangent_light_dir;
}

[shader("vertex")]
VSOutputLight vertex_color_light_main(uint vertex_index: SV_VertexID) {
    Vertex v = push_constants.vertex_reference[vertex_index];
    VSOutputLight output;

    float4x4 world_matrix = push_constants.world_matrix;
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

    return output;
}

[shader("fragment")]
float4 fragment_color_light_main(VSOutputLight input) {
	float4 object_color = input.color;

	float3 light = phong_reflection(input.normal, input.tangent_position, input.tangent_light_dir,
            scene_data.light_color.xyz, input.tangent_view_position, scene_data.ambient_color.xyz);

    return float4(light * object_color.xyz, object_color.a);
}
