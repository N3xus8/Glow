
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
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>, // Unused 
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput,  instance: InstanceInput,) -> @builtin(position) vec4<f32> {

    // reassemble the matrix
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );


    return camera.view_proj * model_matrix * spin.model  * vec4<f32>(in.position, 1.0);
}

 