struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec4f,
    @location(2) tex_coords: vec2f,
    @location(3) normal: vec3f,
    @location(4) n_tex_coords: vec2f,
    @location(5) tangent: vec3f,
    @location(6) bitangent: vec3f,
}

struct InstanceInput {
    @location(7) model_matrix_0: vec4f,
    @location(8) model_matrix_1: vec4f,
    @location(9) model_matrix_2: vec4f,
    @location(10) model_matrix_3: vec4f,
    
    @location(11) normal_matrix_0: vec3f,
    @location(12) normal_matrix_1: vec3f,
    @location(13) normal_matrix_2: vec3f,
}

struct Camera {
    view_pos: vec4f,
    view_proj: mat4x4f
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
    position: vec3f,
    color: vec3f
}

@group(1) @binding(0)
var<uniform> light: Light;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(2)
var t_normal: texture_2d<f32>;
@group(2) @binding(3)
var s_normal: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) n_tex_coords: vec2f,
    @location(2) color: vec4f,
    @location(3) tangent_position: vec3f,
    @location(4) tangent_light_position: vec3f,
    @location(5) tangent_view_position: vec3f
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4f(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );

    let normal_matrix = mat3x3f(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2
    );

    let tangent_matrix = tangent_matrix(normal_matrix, model.normal, model.tangent, model.bitangent);

    let world_position = model_matrix * vec4f(model.position, 1.0);

    var out: VertexOutput;

    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.n_tex_coords = model.n_tex_coords;
    out.color = model.color;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    out.tangent_light_position = tangent_matrix * light.position;

    return out;
}

fn tangent_matrix(normal_matrix: mat3x3f, normal: vec3f, tangent: vec3f, bitangent: vec3f) -> mat3x3f {
    let world_normal = normalize(normal_matrix * normal);
    let world_tangent = normalize(normal_matrix * tangent);
    let world_bitangent = normalize(normal_matrix * bitangent);
    return transpose(mat3x3f(
        world_tangent,
        world_bitangent,
        world_normal
    ));
}

fn tex_map(tex_coords: vec2f, cols: f32, rows: f32, index: f32) -> vec2f {
    let col = (index - 1.0) % cols;
    let row = trunc((index - 1.0) / cols);

    let u = (col + tex_coords.x) / cols;
    let v = (row + tex_coords.y) / rows;

    return vec2f(u, v);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let object_color: vec4f = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4f = textureSample(t_normal, s_normal, in.n_tex_coords);

    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let light_dir = normalize(in.tangent_light_position - in.tangent_position);
    let view_dir = normalize(in.tangent_view_position - in.tangent_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz * in.color.xyz;


    return vec4f(result, in.color.a);
}