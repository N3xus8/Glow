use wgpu::util::DeviceExt;

use crate::utils;

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpinUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    model: [[f32; 4]; 4],
}

impl Default for SpinUniform {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinUniform {
    // initialize
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            model: cgmath::Matrix4::identity().into(),
        }
    }

    // create a buffer uniforn
    pub fn create_spin_uniform_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spin Uniform Buffer"),
            contents: bytemuck::bytes_of(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_from_angle(&mut self, angle: f32) {
        let rotation = cgmath::Matrix4::from_angle_y(cgmath::Rad(angle));

        self.model = rotation.into();
    }

    pub fn bind_group_for_spin_uniform(
        spin_uniform_buffer: &wgpu::Buffer,
        device: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let spin_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("spin_bind_group_layout"),
            });

        let spin_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &spin_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: spin_uniform_buffer.as_entire_binding(),
            }],
            label: Some("spin_bind_group"),
        });

        (spin_bind_group_layout, spin_bind_group)
    }
}

pub struct Spin {
    angle: f32,
    speed: f32, // radians per second
}

impl Spin {
    pub fn new(speed: f32) -> Self {
        Self { angle: 0.0, speed }
    }

    pub fn update(&mut self, dt: f32) {
        self.angle += dt * self.speed;

        // Keep angle bounded
        if self.angle > std::f32::consts::TAU {
            self.angle -= std::f32::consts::TAU;
        }
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }
}
//
// Mirror plane Uniform

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MirrorPlaneUniform {
    normal: [f32; 3], // normal: world-space mirror normal
    _pad1: f32,
    point: [f32; 3], // point: world-space point on the mirror plane
    _pad2: f32,
}

impl MirrorPlaneUniform {
    pub fn new(
        mirror_transform: &cgmath::Matrix4<f32>,
        local_normal: cgmath::Vector3<f32>,
    ) -> MirrorPlaneUniform {
        Self {
            normal: utils::normal_from_transform(mirror_transform, local_normal).into(),
            _pad1: 0.0,
            point: utils::point_from_transform(mirror_transform).into(),
            _pad2: 0.0,
        }
    }

    pub fn create_bind_group_layout(
        device: &wgpu::Device,
        mirror_plane_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let mirror_plane_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mirror_plane_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(
                                std::mem::size_of::<MirrorPlaneUniform>() as u64
                            )
                            .unwrap(),
                        ),
                    },
                    count: None,
                }],
            });

        let mirror_plane_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mirror_plane_bind_group"),
            layout: &mirror_plane_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mirror_plane_buffer.as_entire_binding(),
            }],
        });

        (mirror_plane_bind_group_layout, mirror_plane_bind_group)
    }

    pub fn mirror_plane_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mirror Plane Buffer"),
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}



#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct BlurParams {
   pub direction: [f32; 2],
   pub _padding: [f32; 2], // 8 extra bytes → total 16 bytes
   
}


impl BlurParams {

    pub fn new(x:f32, y: f32) -> Self {
        Self { direction: [x,y], _padding: [0.0; 2] }
    }

    pub fn create_blurparams_uniform_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
            
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("blur params buffer"),
                size: std::mem::size_of::<BlurParams>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
    }
}

    pub fn create_blur_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blur bind group layout"),
            entries: &[
                // Input texture
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
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::Filtering
                    ),
                    count: None,
                },
                // Blur params
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })


    }

    pub fn create_blur_bind_group(
            device: &wgpu::Device, 
            blur_bind_group_layout: &wgpu::BindGroupLayout,
            texture_view: &wgpu::TextureView,
            linear_sampler: &wgpu::Sampler,
            blur_params_buffer: &wgpu::Buffer,
        ) -> wgpu::BindGroup {

        device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("blur bind group"),
            layout: &blur_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&linear_sampler),
                },
                // Blur params uniform
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: blur_params_buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub fn create_linear_sampler(device: &wgpu::Device) -> wgpu::Sampler {

        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,               // IMPORTANT: not a comparison sampler
            anisotropy_clamp: 1,
            border_color: None,
        })
    }



    pub fn create_composite_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("composite bind group layout"),
        entries: &[
            // Scene color
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
            // Outline (hard)
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            // Bloom texture
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(
                    wgpu::SamplerBindingType::Filtering
                ),
                count: None,
            },
        ],
    })
    }


    pub fn create_composite_bind_group(        
            device: &wgpu::Device, 
            composite_bind_group_layout: &wgpu::BindGroupLayout,
            scene_texture_view: &wgpu::TextureView,
            bloom_texture_view: &wgpu::TextureView,
            outline_resolved_texture_view: &wgpu::TextureView,
            linear_sampler: &wgpu::Sampler,
        ) -> wgpu::BindGroup {
    
        device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite bind group"),
                layout: &composite_bind_group_layout,
                entries: &[
                    // Scene color texture
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &scene_texture_view,
                        ),
                    },
                    // Hard outline
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                           outline_resolved_texture_view
                        ),
                    },
                    // Bloom texture (final vertical blur output)
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &bloom_texture_view,
                        ),
                    },
                    // Shared linear sampler
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(
                            &linear_sampler,
                        ),
                    },
                ],
            })

    }

    pub fn create_tone_map_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout{
        
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tone Map Bind Group Layout"),
            entries: &[
                // HDR texture
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
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::Filtering,
                    ),
                    count: None,
                },
            ],
        })

    }

    pub fn create_tone_map_bind_group(
             device: &wgpu::Device, 
             tone_map_bind_group_layout: &wgpu::BindGroupLayout,
             composed_texture_view: &wgpu::TextureView,
             linear_sampler: &wgpu::Sampler,      
    ) -> wgpu::BindGroup {

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Tone Map Bind Group"),
            layout: &tone_map_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &composed_texture_view
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&linear_sampler),
                },
            ],
        })

    }