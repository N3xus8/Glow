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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene   = textureSample(t_scene,   s_sampler, in.uv);
    let outline = textureSample(t_outline, s_sampler, in.uv);
    let bloom   = textureSample(t_bloom,   s_sampler, in.uv);

    // Hard outline compositing (mask-based)


    // Additive bloom
 
 
    //return    scene + bloom * 1. ;
    //return    scene  ;

    let bloom_strength = 3.7 ;

    let scene_rgb = scene.rgb;
    let bloom_rgb = bloom.rgb + vec3(0.0, 0.0, 0.0);

    let color = scene_rgb + bloom_rgb * bloom_strength;

    return vec4(color, 1.0);
    //return vec4(bloom_rgb  * bloom_strength, 1.0);
}
