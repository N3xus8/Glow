@group(0) @binding(0)
var src: texture_2d<f32>;

@group(0) @binding(1)
var dst: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn computeMipMap(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dst_coord = vec2<i32>(gid.xy);

    let dst_size = textureDimensions(dst);
    if (dst_coord.x >= dst_size.x || dst_coord.y >= dst_size.y) {
        return;
    }

    let src_size = textureDimensions(src);

    let base = vec2<i32>(dst_coord * 2);

    let p0 = textureLoad(src, clamp(base, vec2<i32>(0), src_size - 1), 0);
    let p1 = textureLoad(src, clamp(base + vec2<i32>(1, 0), vec2<i32>(0), src_size - 1), 0);
    let p2 = textureLoad(src, clamp(base + vec2<i32>(0, 1), vec2<i32>(0), src_size - 1), 0);
    let p3 = textureLoad(src, clamp(base + vec2<i32>(1, 1), vec2<i32>(0), src_size - 1), 0);

    let color = (p0 + p1 + p2 + p3) * 0.25f;

    textureStore(dst, dst_coord, color);
}
