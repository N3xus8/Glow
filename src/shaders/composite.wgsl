struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;

    // Fullscreen triangle
    let positions = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0),
        vec2( 3.0, -1.0),
        vec2(-1.0,  3.0),
    );

    let pos = positions[idx];
    out.position = vec4(pos, 0.0, 1.0);

    // Convert clip space → UV
    out.uv = pos * 0.5 + vec2(0.5);
    out.uv.y = 1.0 - out.uv.y; // wgpu texture coords

    return out;
}

@group(0) @binding(0) var t_scene: texture_2d<f32>;
@group(0) @binding(1) var t_outline: texture_2d<f32>;
@group(0) @binding(2) var t_bloom: texture_2d<f32>;
@group(0) @binding(3) var s_sampler: sampler;

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let scene   = textureSample(t_scene,   s_sampler, in.uv);
//     let outline = textureSample(t_outline, s_sampler, in.uv);
//     let bloom   = textureSample(t_bloom,   s_sampler, in.uv);

//     // Use ALPHA as the mask
//    let composed = mix(scene, vec4(outline.rgb, 1.0), outline.a);
//     // Add glow
//     //return composed + bloom * 12;
//     return composed + bloom * 2.0;
//     //return outline;
//     //return textureSample(t_scene, s_sampler, in.uv);
//     //return vec4(vec3(outline.a), 1.0);
// }

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene   = textureSample(t_scene,   s_sampler, in.uv);
    let outline = textureSample(t_outline, s_sampler, in.uv);
    let bloom   = textureSample(t_bloom,   s_sampler, in.uv);

    // Hard outline compositing (mask-based)
    let composed =
        scene * (1.0 - outline.a) +
        vec4(outline.rgb, 1.0) * outline.a;

    // Additive bloom
    //return scene + bloom * 2.;
    //return vec4(scene.rgb + bloom.rgb *2., 1.0);
    //return   scene  + outline ;
    //return vec4(vec3(outline.a), 1.0);
    return    scene  + bloom * 2.35 ;
    //return    scene ;
}
