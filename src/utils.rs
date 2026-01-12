// / Load image in texture in native mode . i.e non wasm32.

pub fn load_image(url: &str) -> anyhow::Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
    // Image uses Rayon not available in Wasm

    let path = std::path::Path::new(env!("OUT_DIR")).join("res").join(url);
    Ok(image::open(path)?.flipv().into_rgba8())
}

pub fn create_texture_from_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
) -> anyhow::Result<wgpu::Texture> {
    let width = img.width();
    let height = img.height();

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Image Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // TEXTURE_BINDING is required to use it in shaders
        // COPY_DST is required to copy data into it
        // RENDER_ATTACHMENT is required for copy_external_image_to_texture on some backends
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        view_formats: &[],
    });

    queue.write_texture(
        // Tells wgpu where to copy the pixel data
        wgpu::TexelCopyTextureInfo {
            texture: &texture, // Destination
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img, // RGBA8 bytes
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(img.width() * 4),
            rows_per_image: Some(img.height()),
        },
        texture_size,
    );

    Ok(texture)
}

use cgmath::Vector3;
use winit::window::Icon;

use crate::texture::Texture;

pub fn load_icon(path: &str) -> Icon {
    let img = image::open(path).expect("error opening image").to_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    Icon::from_rgba(rgba, width, height).expect("error convert image to rgba")
}

// Color correction. needed for web browser
pub fn linear_to_srgb(linear: f64) -> f64 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

fn reflection_matrix(
    point: cgmath::Point3<f32>,
    normal: cgmath::Vector3<f32>,
) -> cgmath::Matrix4<f32> {
    use cgmath::EuclideanSpace;
    use cgmath::InnerSpace;

    let n = normal.normalize();
    let d = -n.dot(point.to_vec());

    cgmath::Matrix4::from_cols(
        cgmath::Vector4::new(
            1.0 - 2.0 * n.x * n.x,
            -2.0 * n.x * n.y,
            -2.0 * n.x * n.z,
            0.0,
        ),
        cgmath::Vector4::new(
            -2.0 * n.y * n.x,
            1.0 - 2.0 * n.y * n.y,
            -2.0 * n.y * n.z,
            0.0,
        ),
        cgmath::Vector4::new(
            -2.0 * n.z * n.x,
            -2.0 * n.z * n.y,
            1.0 - 2.0 * n.z * n.z,
            0.0,
        ),
        cgmath::Vector4::new(-2.0 * n.x * d, -2.0 * n.y * d, -2.0 * n.z * d, 1.0),
    )
}

pub fn build_reflection_matrix(
    mirror_transform: &cgmath::Matrix4<f32>,
    local_normal: Vector3<f32>,
) -> cgmath::Matrix4<f32> {
    let mirror_point = point_from_transform(mirror_transform);

    let mirror_normal = normal_from_transform(mirror_transform, local_normal);

    // 2. Build reflection matrix
    reflection_matrix(mirror_point, mirror_normal)
}

pub fn point_from_transform(mirror_transform: &cgmath::Matrix4<f32>) -> cgmath::Point3<f32> {
    use cgmath::{EuclideanSpace, Transform};
    mirror_transform.transform_point(cgmath::Point3::origin())
}

pub fn normal_from_transform(
    mirror_transform: &cgmath::Matrix4<f32>,
    local_normal: Vector3<f32>,
) -> Vector3<f32> {
    use cgmath::InnerSpace;
    use cgmath::Transform;
    mirror_transform.transform_vector(local_normal).normalize()
}

pub fn create_texture_from_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    rgba: &[u8],
    width: u32,
    height: u32,
    label: &str,
) -> Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    Texture {
        texture,
        view,
        sampler,
    }
}

pub fn rgba_f32_to_u8(c: [f32; 4]) -> [u8; 4] {
    fn to_u8(x: f32) -> u8 {
        (x.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    [to_u8(c[0]), to_u8(c[1]), to_u8(c[2]), to_u8(c[3])]
}