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
