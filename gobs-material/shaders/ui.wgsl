struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec4f,
    @location(2) tex_coords: vec2f,
}

struct InstanceInput {
    @location(3) model_matrix_0: vec4f,
    @location(4) model_matrix_1: vec4f,
    @location(5) model_matrix_2: vec4f,
    @location(6) model_matrix_3: vec4f,
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
    @location(1) color: vec4f,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4f(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );

    let world_position = model_matrix * vec4f(model.position, 1.0);

    var out: VertexOutput;

    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.color = model.color;

    return out;
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

    return in.color * object_color;
}