struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BlurParams {
    direction: vec2<f32>, // (1,0) = horizontal, (0,1) = vertical
};

@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BlurParams;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0),
        vec2( 3.0, -1.0),
        vec2(-1.0,  3.0),
    );

    let pos = positions[idx];
    var out: VertexOutput;
    out.position = vec4(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + 0.5;
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let size_u = textureDimensions(t_input);
    let size = vec2<f32>(f32(size_u.x), f32(size_u.y));
    let texel = params.direction * 2.3 / size;

    var sum = vec4<f32>(0.0);

    sum += textureSample(t_input, s_sampler, in.uv - 4.0 * texel) * 0.05;
    sum += textureSample(t_input, s_sampler, in.uv - 2.0 * texel) * 0.09;
    sum += textureSample(t_input, s_sampler, in.uv)              * 0.62;
    sum += textureSample(t_input, s_sampler, in.uv + 2.0 * texel) * 0.09;
    sum += textureSample(t_input, s_sampler, in.uv + 4.0 * texel) * 0.05;

    return sum;
}
