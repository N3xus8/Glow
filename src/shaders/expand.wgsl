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
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert from clip space to NDC
    let ndc = in.position.xyz / in.position.w;

    // Optional: scale noise frequency
    let noise_scale = 5.0;
    let noise = simplex_noise_3d(ndc * noise_scale); 

    // Map noise from [-1,1] → [0,1] if needed
    let noise01 = noise * 0.5 + 0.5;
   
    // Subtle edge modulation
    //let noisy_edge = clamp(noise01, 0.0, 1.0);
    let noisy_edge = clamp(noise * 0.2 + 0.8, 0.0, 1.0);
    //let alpha = noisy_edge;
    let alpha = 1.0 ;
    let edge_color = vec3<f32>(0.0, 0.5, 0.5); //* alpha; // black edge

    // let threshold = 0.5;
    // let noisy_threshold = threshold + (noise - 0.5) * 0.5;
    // let edge_mask = step(noisy_threshold, 0.4);


    return vec4<f32>(edge_color, alpha) ; // black outline
    //return vec4<f32>(edge_color, edge_mask);
}



fn simplex_noise_3d(v: vec3<f32>) -> f32 {
    let C = vec2(1. / 6., 1. / 3.);
    let D = vec4(0., 0.5, 1., 2.);

    // first corner
    var i = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);

    // other corners
    let g = step(x0.yzx, x0.xyz);
    let l = 1. - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    // x0 = x0 - 0. + 0. * C
    let x1 = x0 - i1 + 1. * C.xxx;
    let x2 = x0 - i2 + 2. * C.xxx;
    let x3 = x0 - 1. + 3. * C.xxx;

    // permutations
    i = i % vec3(289.);
    let p = permute_4_(permute_4_(permute_4_(
        i.z + vec4(0., i1.z, i2.z, 1.)) +
        i.y + vec4(0., i1.y, i2.y, 1.)) +
        i.x + vec4(0., i1.x, i2.x, 1.)
    );

    // gradients (NxN points uniformly over a square, mapped onto an octahedron)
    let n_ = 1. / 7.; // N=7
    let ns = n_ * D.wyz - D.xzx;

    let j = p - 49. * floor(p * ns.z * ns.z); // mod(p, N*N)

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7. * x_); // mod(j, N)

    let x = x_ * ns.x + ns.yyyy;
    let y = y_ * ns.x + ns.yyyy;
    let h = 1. - abs(x) - abs(y);

    let b0 = vec4(x.xy, y.xy);
    let b1 = vec4(x.zw, y.zw);

    let s0 = floor(b0) * 2. + 1.;
    let s1 = floor(b1) * 2. + 1.;
    let sh = -step(h, vec4(0.));

    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3(a0.xy, h.x);
    var p1 = vec3(a0.zw, h.y);
    var p2 = vec3(a1.xy, h.z);
    var p3 = vec3(a1.zw, h.w);

    // normalize gradients
    let norm = taylor_inv_sqrt_4_(vec4(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 = p0 * norm.x;
    p1 = p1 * norm.y;
    p2 = p2 * norm.z;
    p3 = p3 * norm.w;

    // mix final noise value
    var m = 0.5 - vec4(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
    m = max(m, vec4(0.));
    m *= m;
    return 105. * dot(m * m, vec4(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}

fn permute_4_(x: vec4<f32>) -> vec4<f32> {
    return ((x * 34. + 1.) * x) % vec4<f32>(289.);
}


fn taylor_inv_sqrt_4_(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}
