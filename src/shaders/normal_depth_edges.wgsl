

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
  

    var out: VertexOutput;
    // Hardcoded positions for 3 vertices forming a massive triangle
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1., -1.),
        vec2<f32>( 3., -1.),
        vec2<f32>(-1.,  3.)
    );
    
    let xy = pos[idx];
    out.clip_position = vec4<f32>(xy, 0.0, 1.0);
     out.uv = xy * 0.5 + 0.5;
     out.uv.y = 1.0 - out.uv.y; // Flip Y for WGPU texture coordinates
    return out;
}

@group(0) @binding(0) var t_color: texture_2d<f32>;
@group(0) @binding(1) var s_sampler: sampler;
@group(0) @binding(2) var t_depth: texture_depth_2d;
@group(0) @binding(3) var t_normal: texture_2d<f32>;

// Linearize depth so edges are consistent regardless of distance
fn linearize_depth(depth: f32) -> f32 {
    let z = depth * 2.0 - 1.0;
    let near = 0.1;
    let far = 100.0;
    return (2.0 * near * far) / (far + near - z * (far - near));
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(t_color));
    let offset = 1.0 / tex_size;

    /* ------------------ DEPTH ------------------ */

    let d_c = linearize_depth(textureSample(t_depth, s_sampler, in.uv));
    let d_l = linearize_depth(textureSample(t_depth, s_sampler, in.uv + vec2<f32>(-offset.x, 0.0)));
    let d_r = linearize_depth(textureSample(t_depth, s_sampler, in.uv + vec2<f32>( offset.x, 0.0)));
    let d_u = linearize_depth(textureSample(t_depth, s_sampler, in.uv + vec2<f32>(0.0,  offset.y)));
    let d_d = linearize_depth(textureSample(t_depth, s_sampler, in.uv + vec2<f32>(0.0, -offset.y)));

    let depth_diff =
          abs(d_c - d_l)
        + abs(d_c - d_r)
        + abs(d_c - d_u)
        + abs(d_c - d_d);

    let depth_edge = smoothstep(0.002, 0.006, depth_diff* 0.03);

    /* ------------------ NORMAL ------------------ */

    let n_c = normalize(textureSample(t_normal, s_sampler, in.uv).xyz);
    let n_l = normalize(textureSample(t_normal, s_sampler, in.uv + vec2<f32>(-offset.x, 0.0)).xyz);
    let n_r = normalize(textureSample(t_normal, s_sampler, in.uv + vec2<f32>( offset.x, 0.0)).xyz);
    let n_u = normalize(textureSample(t_normal, s_sampler, in.uv + vec2<f32>(0.0,  offset.y)).xyz);
    let n_d = normalize(textureSample(t_normal, s_sampler, in.uv + vec2<f32>(0.0, -offset.y)).xyz);

    let normal_diff =
          (1.0 - dot(n_c, n_l))
        + (1.0 - dot(n_c, n_r))
        + (1.0 - dot(n_c, n_u))
        + (1.0 - dot(n_c, n_d));

    let normal_edge = smoothstep(0.01, 0.15, normal_diff*7.0);

    /* ------------------ COMBINE ------------------ */

    let edge = max(depth_edge * 0.6, normal_edge * 1.2);

    let base_color = textureSample(t_color, s_sampler, in.uv);
    let edge_color = vec4<f32>(0.6, 0.7, 0.7, 1.0);

    //return vec4(vec3(normal_edge), 1.0);
    //return vec4(vec3(normal_diff * 5.0), 1.0);

    //return vec4(vec3(depth_edge), 1.0);
    return mix(base_color, edge_color, edge);

}

