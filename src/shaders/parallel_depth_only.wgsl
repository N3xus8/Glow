// Vertex shader
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
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>, // local normals
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>, // Passed to fragment shader
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

// struct FragmentOutput {
//     @location(0) color: vec4<f32>,
//     @location(1) normal: vec4<f32>,
// }

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {

    var out: VertexOutput;
    // reassemble the matrix
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // Use only the Model Matrix to get World Space
    // We convert to mat3x3 to strip away any translation data
    out.world_normal = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz
    ) * model.normal;


    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * spin.model * vec4<f32>(model.position, 1.0);
    return out;
}
 //  

// Fragment shader
@fragment
fn fs_main(){
// no color

}
 