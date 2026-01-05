// expand object

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct SpinUniform {
    model:  mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> spin: SpinUniform;



struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(2) normal   : vec3<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position : vec4<f32>,
};

const OUTLINE_THICKNESS : f32 = 0.07;

@vertex
fn vs_main(input: VertexInput,  instance: InstanceInput,) -> VertexOutput {
    var out : VertexOutput;

    // reassemble the matrix
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let expanded = input.position + input.normal * OUTLINE_THICKNESS;

    out.position = camera.view_proj * model_matrix * spin.model   * vec4<f32>(expanded, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0); // black outline
}