struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec3f
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4f,
    @location(3) model_matrix_1: vec4f,
    @location(4) model_matrix_2: vec4f,
    @location(5) model_matrix_3: vec4f,
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

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f
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
    out.color = model.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let color = light.color * in.color;
    return vec4f(color, 1.0);
}