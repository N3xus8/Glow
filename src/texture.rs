use anyhow::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::utils::{load_image, create_texture_from_image};
#[cfg(target_arch = "wasm32")]
use crate::web_utils::load_texture_from_image_web;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;


pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub async fn get_texture_from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        url: &str,
    ) -> Result<Self> {
        #[cfg(target_arch = "wasm32")]
        let texture = load_texture_from_image_web(device, queue, url)
            .await
            .map_err(|e| log::error!("texture error {:?} ", e))
            .unwrap_throw();

        #[cfg(not(target_arch = "wasm32"))]
        let img = load_image(url)?;
        #[cfg(not(target_arch = "wasm32"))]
        let texture = create_texture_from_image(device, queue, img)?;

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            //mipmap_filter: wgpu::FilterMode::Nearest, // 27.0
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn bind_group_for_texture(
        &self,
        device: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        (texture_bind_group_layout, diffuse_bind_group)
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const DEPTH_STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            //mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }


    // /  N O R M A L  T E X T U R E 
    // /
    pub fn create_normal_texture(      
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
        sample_count: u32,
    ) -> Self {

        
        let usage = if sample_count == 1 {           
            wgpu::TextureUsages::RENDER_ATTACHMENT |  wgpu::TextureUsages::TEXTURE_BINDING 
        } else {
            wgpu::TextureUsages::RENDER_ATTACHMENT 
        };

        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        // 1. Normal Texture: Stores XYZ vectors. 
        // We use >>>Rgba16Float<<< for high precision (avoids jagged edges on curves).
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float, 
            usage: usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());


        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Edge Detection Sampler"),
            // Clamp to edge prevents the "border" of the screen from 
            // bleeding into the edges when sampling neighbors
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // Use Linear if you want softer, smoother edges.
            // Use Nearest if you want pixel-perfect, sharp "hard" edges.
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            compare: None,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }

    }

    pub fn normal_group_for_texture(
        &self,
        device: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {

       let normal_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("normal_bind_group_layout"),
            }); 

        let normal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &normal_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("normal_bind_group"),
        });

        (normal_bind_group_layout, normal_bind_group)              
    
    }

}
    




pub fn create_multisampled_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: config.width.max(1),
        height: config.height.max(1),
        depth_or_array_layers: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        #[cfg(not(target_arch = "wasm32"))]
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        #[cfg(target_arch = "wasm32")]
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    };

    device
        .create_texture(multisampled_frame_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default())
}




// /
// /  C O L O R   T E X T U R E 
// /
// / Intermediate/offscreen texture 
pub struct ColorTexture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl ColorTexture {
    pub fn create_color_texture(
            device: &wgpu::Device,
            config: &wgpu::SurfaceConfiguration,
            label: &str,
            sample_count: u32,
            is_hdr: bool,
        ) 
        -> Self {

        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let texture_format = if is_hdr {wgpu::TextureFormat::Rgba16Float} else {config.format };
        let usage = if sample_count == 1 {wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING} else {wgpu::TextureUsages::RENDER_ATTACHMENT};
        // This is your internal "canvas"
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count, 
            dimension: wgpu::TextureDimension::D2,
            // Use the same format as your surface (e.g., Bgra8UnormSrgb)
            //format: config.format,
            format: texture_format ,
            // USAGE IS KEY: 
            // RENDER_ATTACHMENT so we can draw to it in Pass 1.
            // TEXTURE_BINDING so the Edge Shader can read it in Pass 2.
            usage: usage,
            //usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }
}