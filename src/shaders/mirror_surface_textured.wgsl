//

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

@group(0) @binding(0)
var mirror_texture: texture_2d<f32>;

@group(0) @binding(1)
var mirror_sampler: sampler;


// Vertex shader

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
        // reassemble the matrix
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    out.clip_position =  camera.view_proj * model_matrix * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(mirror_texture, mirror_sampler, in.uv);
}
