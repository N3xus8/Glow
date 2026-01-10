use anyhow::*;

use crate::model::{InstanceRaw, ModelVertex, Vertex};
pub struct Pipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn build_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        camera_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        spin_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        is_hdr: bool,
    ) -> Result<Pipeline> {

        let texture_format = if is_hdr {wgpu::TextureFormat::Rgba16Float} else {config.format };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Scene Render Pipeline Layout"),
                bind_group_layouts: &[
                    texture_bind_group_layout,
                    camera_uniform_bind_group_layout,
                    spin_uniform_bind_group_layout,
                ],
                immediate_size: 0,
            });

        //Pipeline

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Scene Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    //format: config.format,
                    format: texture_format,
                    //blend: None,
                    //blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                    Some(wgpu::ColorTargetState { // Location 1
                        format: texture_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }), 
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                //stencil: wgpu::StencilState::default(),
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0xFF,
                    write_mask: 0x00,
                },

                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: sample_count,              // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview_mask: None, // 5.
            cache: None,
            // multiview: None,     // version 27.0
        });
        Ok(Self {
            pipeline: render_pipeline,
        })
    }

       pub fn mask_render_pipeline(
        device: &wgpu::Device,
        camera_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        spin_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        sample_count: u32,
    ) -> Result<Pipeline> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("stencil"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/stencil.wgsl").into()),
        });

        let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mask_pipeline_layout"),
            bind_group_layouts: &[camera_uniform_bind_group_layout, spin_uniform_bind_group_layout],
            immediate_size: 0,
            //push_constant_ranges: &[],
        });

        let mask_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mask_Render_Pipeline"),
            layout: Some(&mask_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Replace,
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0xFF,
                    write_mask: 0xFF,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview_mask: None,
            //multiview: None,
            cache: None,
        });

        Ok(Self {
            pipeline: mask_render_pipeline,
        })
    }


    pub fn outline_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        spin_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        sample_count: u32,
        is_hdr: bool,
    ) -> Result<Pipeline> {

        let texture_format = if is_hdr {wgpu::TextureFormat::Rgba16Float} else {config.format };


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("stencil_expand"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/expand.wgsl").into()),
        });

        let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("outline_pipeline_layout"),
            bind_group_layouts: &[camera_uniform_bind_group_layout, spin_uniform_bind_group_layout],
            immediate_size: 0,
            //push_constant_ranges: &[],
        });
        let outline_color_target = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let outline_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("outline_Render_Pipeline"),
            layout: Some(&mask_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    //format: config.format,
                    format: texture_format,
                    //blend: Some(wgpu::BlendState::REPLACE),
                    blend: Some(outline_color_target),
                    //blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back), // important
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState {
                        front: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::NotEqual,
                            pass_op: wgpu::StencilOperation::Keep,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                        },
                        back : wgpu::StencilFaceState::IGNORE,
                        read_mask: 0xFF,
                        write_mask: 0x00,
                    },
                    bias: wgpu::DepthBiasState {
                        constant: 1, // helps z-fighting
                        slope_scale: 1.0,
                        clamp: 0.0,
                    },
                }),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview_mask: None,
            //multiview: None,
            cache: None,

        });

        Ok(Self {
            pipeline: outline_pipeline,
        })

    }

// /  B L U R 

  pub fn blur_pipeline (
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        blur_bind_group_layout:wgpu::BindGroupLayout,
        is_hdr: bool,
        ) -> Result<Pipeline> {

        let texture_format = if is_hdr {wgpu::TextureFormat::Rgba16Float} else {config.format };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blur shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/blur.wgsl").into()),
        });

        let blur_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blur pipeline layout"),
            bind_group_layouts: &[&blur_bind_group_layout],
            immediate_size: 0,
        });

        let blur_pipeline =device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("outline blur pipeline"),
                layout: Some(&blur_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[], // fullscreen triangle
                    compilation_options: wgpu::PipelineCompilationOptions::default(),

                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        //format: config.format,
                        format: texture_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
            });

        Ok(Self { pipeline: blur_pipeline })

    }

pub fn edge_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    edge_bind_group_layout: &wgpu::BindGroupLayout,
    is_hdr: bool,
    ) -> Result<Pipeline> {

    let texture_format = if is_hdr {wgpu::TextureFormat::Rgba16Float} else {config.format };

    let edge_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("normal depth edge shader"),
           source: wgpu::ShaderSource::Wgsl(include_str!("shaders/normal_depth_edges.wgsl").into()),
        });

    let edge_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Edge Pipeline Layout"),
        bind_group_layouts: &[&edge_bind_group_layout ],
        immediate_size: 0, 
    });

    let edge_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Edge Detection Pipeline"),
        layout: Some(&edge_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &edge_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &edge_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: texture_format, 
                //format: wgpu::TextureFormat::Rgba16Float,
                // Use Alpha Blending if you want to overlay edges on the scene
                blend: None,
                //blend: Some(wgpu::BlendState::ALPHA_BLENDING), 
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None, // No depth testing needed for a full-screen quad
        multisample: wgpu::MultisampleState::default(), // 1-sample because this is post-render
        multiview_mask: None,
        cache: None,
        });

        Ok(Self {
            pipeline: edge_pipeline,
        })


    }

  pub fn composite_pipeline (
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        composite_bind_group_layout:wgpu::BindGroupLayout,
        is_hdr: bool, 
        ) -> Result<Pipeline> {

        let texture_format = if is_hdr {wgpu::TextureFormat::Rgba16Float} else {config.format };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("composite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/composite.wgsl").into()),
        });


        let composite_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("composite pipeline layout"),
            bind_group_layouts: &[&composite_bind_group_layout],
            immediate_size: 0,
        });


        let composite_pipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("composite pipeline"),
            layout: Some(&composite_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(), // fullscreen triangle
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    //format: config.format, // swapchain format
                    format: texture_format,
                    blend: None,
                    //blend: Some(wgpu::BlendState::REPLACE),
                    //blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self { pipeline: composite_pipeline })
    }

    pub fn tone_map_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        tone_map_bind_group_layout:wgpu::BindGroupLayout,

    ) -> Result<Pipeline> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tone map shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/hdr_lite.wgsl").into()),
        });


        let tone_map_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tone map pipeline layout"),
            bind_group_layouts: &[&tone_map_bind_group_layout],
            immediate_size: 0,
        });

        let tone_map_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tone Map Pipeline"),
            layout: Some(&tone_map_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format, // Bgra8UnormSrgb
                    blend: None,            // IMPORTANT
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        
        Ok(Self{ pipeline: tone_map_pipeline})
    }

    pub fn parallel_depth_pipeline(
        device: &wgpu::Device,
        camera_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        spin_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<Pipeline> {

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("parallel depth only"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/parallel_depth_only.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("parallel depth only Pipeline Layout"),
                bind_group_layouts: &[
                    camera_uniform_bind_group_layout,
                    spin_uniform_bind_group_layout,
                ],
                immediate_size: 0,
              //push_constant_ranges: &[], // older version
            });

        //Pipeline

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,              
                mask: !0,                         
                alpha_to_coverage_enabled: false, 
            },
            multiview_mask: None, 
            cache: None,
        });
        Ok(Self {
            pipeline: render_pipeline,
        })       
    }
}