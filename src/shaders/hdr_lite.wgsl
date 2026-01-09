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

@group(0) @binding(0) var t_hdr: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let hdr = textureSample(t_hdr, s, in.uv).rgb;

    // // Simple Reinhard tone mapping
    // let mapped = hdr / (hdr + vec3(1.0));

    // // Output to sRGB surface
    // return vec4(mapped, 1.0);
    let hdr = textureSample(t_hdr, s, in.uv);
    let sdr = aces_tone_map(hdr.rgb);
    //return vec4(sdr, hdr.a);
    return vec4(sdr, 1.0);
}


// Maps HDR values to linear values
// Based on http://www.oscars.org/science-technology/sci-tech-projects/aces
fn aces_tone_map(hdr: vec3<f32>) -> vec3<f32> {
    let m1 = mat3x3(
        0.59719, 0.07600, 0.02840,
        0.35458, 0.90834, 0.13383,
        0.04823, 0.01566, 0.83777,
    );
    let m2 = mat3x3(
        1.60475, -0.10208, -0.00327,
        -0.53108,  1.10813, -0.07276,
        -0.07367, -0.00605,  1.07602,
    );
    let v = m1 * hdr;
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), vec3(0.0), vec3(1.0));
}

fn _aces_tone_map(v: vec3<f32>) -> vec3<f32> {

    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    
    return saturate((v * (a * v + b)) / (v * (c * v + d) + e));
}