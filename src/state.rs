use std::sync::Arc;

use instant::Instant;
use wgpu::BindGroup;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{camera::{Camera, CameraController, CameraUniform, bind_group_for_camera_uniform, create_camera_buffer}, depth_stencil::{self, StencilTexture}, extra::{self, BlurParams, Spin, SpinUniform, create_blur_bind_group, create_blur_bind_group_layout, create_composite_bind_group_layout, create_linear_sampler}, model::{DrawModel, Instance, Model, create_instance_buffer}, pipeline::Pipeline, resources, texture::{ColorTexture, create_multisampled_view}};
use crate::extra::create_composite_bind_group;

pub struct State {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub render_pipeline: wgpu::RenderPipeline,
        is_paused: bool,
    pub diffuse_bind_group: wgpu::BindGroup,
    obj_model: Model,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    last_frame: Instant,
    spin: Spin,
    spin_uniform: SpinUniform,
    spin_buffer: wgpu::Buffer,
    spin_bind_group: wgpu::BindGroup,
    depth_stencil: StencilTexture,
    stencil_pipeline: wgpu::RenderPipeline,
    sample_count: u32,
    color_texture: Option<ColorTexture>,
    resolve_texture: Option<ColorTexture>,
    outline_pipeline: wgpu::RenderPipeline,
    blur_params_uniform_buffer: wgpu::Buffer,
    blur_outline_resolved_bind_group: Option<BindGroup>,
    blur_inter_bind_group: Option<BindGroup>,
    blur_intermediate_texture: Option<ColorTexture>,
    outline_bloom_texture: Option<ColorTexture>,
    linear_sampler: wgpu::Sampler,
    blur_pipeline: wgpu::RenderPipeline,
    scene_color_texture: Option<ColorTexture>,
    composite_bind_group: Option<BindGroup>,
    composite_pipeline: wgpu::RenderPipeline,
    pub window: Arc<Window>,


}




impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // Surface
        let surface = instance.create_surface(window.clone())?;

        // Adapter

        let adapter = if cfg!(target_arch = "wasm32") {
            instance //  TODO select the most relevant adapter
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await?
        } else {
            instance // TODO select the most relevant adapter
                .enumerate_adapters(wgpu::Backends::all())
                .await
                .into_iter()
                .find(|adapter| adapter.is_surface_supported(&surface))
                .ok_or(anyhow::anyhow!("No adapter found"))?
        };

        // Device & Queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // /
        let sample_count: u32 = 4;

        // /  
        // /
        // / C U B E   M O D E L  

        //  Texture for Cube
        let url = "images/wgpu-logo.png";
        let diffuse_texture =
            crate::texture::Texture::get_texture_from_image(&device, &queue, url).await?;

        let (diffuse_bind_group_layout, diffuse_bind_group) =
            diffuse_texture.bind_group_for_texture(&device);  

        // Load Mesh for Cube
        let obj_model = resources::load_model(
            "models/cube.obj",
            &device,
            &queue,
            &diffuse_bind_group_layout,
        )
        .await
        .unwrap();

        // /
        // / I N S T A N C E S
        // /

        let instances = Instance::generate_instances();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = create_instance_buffer(&device, &instance_data);

        // / C A M E R A
        // /
        let camera = Camera::new(
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            (0.0, 1.0, 2.0),
            // have it look at the origin
            (0.0, 0.0, 0.0),
            // which way is "up"
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = create_camera_buffer(&camera_uniform, &device);
        let (camera_bind_group_layout, camera_bind_group) =
            bind_group_for_camera_uniform(&camera_buffer, &device);

        let camera_controller = CameraController::new(0.1);

        // / D E P T H   T E X T U R E 
        // /

        // let depth_texture = crate::texture::Texture::create_depth_texture(&device, &config, "depth_texture");



        // / S T E N C I L  T E X T U R E
        // /

        let depth_stencil = depth_stencil::StencilTexture::create_stencil_texture(
            &device,
            &config,
            "depth_stencil",
            sample_count,
        );

        // / C O L O R  T E X T U R E 

        //let color_texture = ColorTexture::create_color_texture(&device, &config, "Color Texture", sample_count);
        let color_texture = None;

        // /  R E S O L V E   T E X T U R E 
        // /
        let resolve_texture = None;

        // / S P I N

        let last_frame = Instant::now();
        let spin = Spin::new(1.5);
        let spin_uniform = SpinUniform::new();
        let spin_buffer = spin_uniform.create_spin_uniform_buffer(&device);
        let (spin_bind_group_layout, spin_bind_group) =
            SpinUniform::bind_group_for_spin_uniform(&spin_buffer, &device);

        // /  B L U R  F O R  G L O W 

        // // (1,0) = horizontal, (0,1) = vertical

        let blur_params = BlurParams::new(1., 0.);

        let blur_params_uniform_buffer = blur_params.create_blurparams_uniform_buffer(&device);

        let linear_sampler = create_linear_sampler(&device);

        let blur_outline_resolved_bind_group = None;

        let blur_inter_bind_group = None;

        let blur_intermediate_texture = None;

        let outline_bloom_texture = None;

        let scene_color_texture= None;

        let composite_bind_group = None;
        // /
        // /      P I P E L I N E S
        // /

        // Pipeline
        let pipeline_struct = Pipeline::build_render_pipeline(
            &device,
            &config,
            sample_count,
            &diffuse_bind_group_layout,
            &camera_bind_group_layout,
            &spin_bind_group_layout,
        )?;
        let render_pipeline = pipeline_struct.pipeline;        

        // Stencil Pipeline
        let stencil_pipeline_struct =
            Pipeline::mask_render_pipeline(&device, &camera_bind_group_layout, &spin_bind_group_layout, sample_count)?;

        let stencil_pipeline = stencil_pipeline_struct.pipeline;


        // Outline Stencil pipeline

        let outline_pipeline_struct =
            Pipeline::outline_pipeline(&device, &config, &camera_bind_group_layout, &spin_bind_group_layout, sample_count)?;

        let outline_pipeline = outline_pipeline_struct.pipeline;

        // Blur pipeline 

        let blur_bind_group_layout = create_blur_bind_group_layout(&device);

        let blur_pipeline_struct = Pipeline::blur_pipeline(&device, &config, blur_bind_group_layout)?;
        let blur_pipeline = blur_pipeline_struct.pipeline;

        // Composite pipeline 

        let composite_bind_group_layout = create_composite_bind_group_layout(&device);
        let composite_pipeline_struct = Pipeline::composite_pipeline(&device, &config, composite_bind_group_layout)?;
        let composite_pipeline =composite_pipeline_struct.pipeline;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            is_paused: false,
            diffuse_bind_group,
            obj_model,
            instances,
            instance_buffer,
            camera,
            camera_uniform,
            camera_bind_group,
            camera_buffer,
            camera_controller,
            depth_stencil,
            stencil_pipeline,
            last_frame,
            spin,
            spin_uniform,
            spin_bind_group,
            spin_buffer,
            sample_count,
            color_texture,
            resolve_texture,
            outline_pipeline,
            blur_params_uniform_buffer,
            blur_outline_resolved_bind_group,
            blur_inter_bind_group,
            blur_intermediate_texture,
            outline_bloom_texture,
            linear_sampler,
            blur_pipeline,
            scene_color_texture,
            composite_bind_group,
            composite_pipeline,
            window,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
                if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            self.camera.aspect = width as f32 / height as f32;

            // This is a fix from chatgpt otherwise it only works for desktop not for browser.
            self.camera_uniform.update_view_proj(&self.camera);
            //self.depth_texture = crate::texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            
            self.depth_stencil = depth_stencil::StencilTexture::create_stencil_texture(
                &self.device,
                &self.config,
                "depth_stencil",
                self.sample_count,
            );

            self.color_texture = Some(ColorTexture::create_color_texture(
                &self.device, 
                &self.config, 
                "Color Texture",
                self.sample_count));

            self.resolve_texture = Some(ColorTexture::create_color_texture(&self.device, &self.config, "Resolve Color  Texture", 1)); 
            
            self.blur_intermediate_texture = Some(ColorTexture::create_color_texture(&self.device, &self.config, "Blur Intermediate Color Texture", 1));
            
            self.outline_bloom_texture  = Some(ColorTexture::create_color_texture(&self.device, &self.config, "Bloom Color Texture", 1));
            
            self.scene_color_texture  = Some(ColorTexture::create_color_texture(&self.device, &self.config, "Scene Color Texture", 1));

            let blur_bind_group_layout = create_blur_bind_group_layout(&self.device);

            let blur_resolve_texture_view = if self.sample_count == 1 
            {
                &self.color_texture.as_ref().ok_or("cannot get texture").unwrap().view
            }
            else {
                &self.resolve_texture.as_ref().ok_or("cannot get texture").unwrap().view
            };

            self.blur_outline_resolved_bind_group = Some(create_blur_bind_group(
                &self.device, 
                &blur_bind_group_layout, 
                blur_resolve_texture_view,
                &self.linear_sampler, 
                &self.blur_params_uniform_buffer
            ));

            self.blur_inter_bind_group = Some(create_blur_bind_group(
                &self.device, 
                &blur_bind_group_layout, 
                &self.blur_intermediate_texture.as_ref().ok_or("cannot get texture").unwrap().view,
                &self.linear_sampler, 
                &self.blur_params_uniform_buffer
            ));


        // / C O M P O S I T E 

        let composite_bind_group_layout = create_composite_bind_group_layout(&self.device);

        self.composite_bind_group = Some(create_composite_bind_group(
            &self.device, 
            &composite_bind_group_layout, 
            &self.scene_color_texture.as_ref().ok_or("cannot get texture").unwrap().view, 
            &self.outline_bloom_texture.as_ref().ok_or("cannot get texture").unwrap().view,
            &self.resolve_texture.as_ref().ok_or("cannot get texture").unwrap().view,
            &self.linear_sampler));


        }
    }

    pub fn update(&mut self) {
                // Delta time
        let now = Instant::now();
        let mut dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Clamp for browser tab resume
        dt = dt.min(0.1);

        if !self.is_paused {
            // Update logic
            self.spin.update(dt);

            // Update GPU data
            self.spin_uniform.update_from_angle(self.spin.angle());

            self.queue.write_buffer(
                &self.spin_buffer,
                0,
                bytemuck::bytes_of(&[self.spin_uniform]),
            );
        }
        // Camera
        self.camera_controller.update_camera(&mut self.camera);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        // Framebuffer / swapchain
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());


        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Camera uniform normal mode (for non-reflected mode)

        self.camera_uniform.update_view_proj(&self.camera);

        // / W R I T E  C A M E R A  B U F F E R 

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        
        // /
        //
        // S T E N C I L   P A S S
        //


        let mut stencil_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("stencil pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_stencil.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: wgpu::StoreOp::Store,
                }),
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        stencil_pass.set_stencil_reference(1);
        stencil_pass.set_pipeline(&self.stencil_pipeline);
        stencil_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        stencil_pass.set_bind_group(1, &self.spin_bind_group, &[]);
        stencil_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        stencil_pass.draw_mesh_instanced(&self.obj_model.meshes[0], 0..self.instances.len() as u32);

        drop(stencil_pass);

        // /
        // /   O U T L I N E   S T E N C I L
        // /


        let outline_pass_color_attachments = if self.sample_count == 1 
            {
                wgpu::RenderPassColorAttachment {
                    view: &self.color_texture.as_ref().ok_or(wgpu::SurfaceError::Lost)?.view,
                    //view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color{ 
                            r: 0.67, 
                            g: 0.27, 
                            b: 0.15, 
                            a: 1. }), // <- DO NOT CLEAR
                        store: wgpu::StoreOp::Store,
                    },
                }
            } else {
                wgpu::RenderPassColorAttachment {
                    view: &self.color_texture.as_ref().ok_or(wgpu::SurfaceError::Lost)?.view,
                    //view: &view,
                    depth_slice: None,
                    resolve_target: Some(&self.resolve_texture.as_ref().ok_or(wgpu::SurfaceError::Lost)?.view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color{ 
                            r: 0.67, 
                            g: 0.27, 
                            b: 0.15, 
                            a: 1. }), // <- DO NOT CLEAR
                        store: wgpu::StoreOp::Store,
                    },
                }
            };
        

        let outline_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_stencil.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.), // <- clear depth again
                    store: wgpu::StoreOp::Store,
                }),

                stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load, // <- keep mask
                store: wgpu::StoreOp::Store,
                }) ,
            };

        let mut outline_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("outline Pass"),
                color_attachments: &[Some(outline_pass_color_attachments)],
                depth_stencil_attachment: Some(outline_depth_stencil_attachment),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
        });

        outline_pass.set_pipeline(&self.outline_pipeline);
        outline_pass.set_stencil_reference(1);
        outline_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        outline_pass.set_bind_group(1, &self.spin_bind_group, &[]);
        outline_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        outline_pass.draw_mesh_instanced(&self.obj_model.meshes[0], 0..self.instances.len() as u32);

        drop(outline_pass);

        // /
        //  B L U R   O U T L I N E S

        // Horizontal blur
        self.queue.write_buffer(
            &self.blur_params_uniform_buffer,
            0,
            bytemuck::bytes_of(&BlurParams {
                direction: [1.0, 0.0],
            }),
        );

        let blur_horizontal_pass_color_attachments = 
                wgpu::RenderPassColorAttachment {
                    view: &self.blur_intermediate_texture.as_ref().ok_or(wgpu::SurfaceError::Lost)?.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                };
    
        let mut blur_horizontal_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("horizontal blur"),
            color_attachments: &[Some(blur_horizontal_pass_color_attachments)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        blur_horizontal_pass.set_pipeline(&self.blur_pipeline);
        blur_horizontal_pass.set_bind_group(0, self.blur_outline_resolved_bind_group.as_ref().ok_or(wgpu::SurfaceError::Lost)?, &[]);
        blur_horizontal_pass.draw(0..3, 0..1);

        drop(blur_horizontal_pass);

        // Vertical blur
        self.queue.write_buffer(
            &self.blur_params_uniform_buffer,
            0,
            bytemuck::bytes_of(&BlurParams {
                direction: [0.0, 1.0],
            }),
        );

        let blur_vertical_pass_color_attachments = 
                    wgpu::RenderPassColorAttachment {
                        view: &self.outline_bloom_texture.as_ref().ok_or(wgpu::SurfaceError::Lost)?.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                    },
                };
        
        let mut blur_vertical_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("vertical blur"),
            color_attachments: &[Some(blur_vertical_pass_color_attachments)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        blur_vertical_pass.set_pipeline(&self.blur_pipeline);
        blur_vertical_pass.set_bind_group(0, self.blur_inter_bind_group.as_ref().ok_or(wgpu::SurfaceError::Lost)?, &[]);
        blur_vertical_pass.draw(0..3, 0..1);

        drop(blur_vertical_pass);


        // /
        // T O T A L  S C E N E
        // /


        
        //
        let render_pass_color_attachments = if self.sample_count == 1
             {  
                wgpu::RenderPassColorAttachment {
                    //view: &view,
                    view: &self.scene_color_texture.as_ref().ok_or(wgpu::SurfaceError::Lost)?.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }

            } else {
                wgpu::RenderPassColorAttachment {
                    view: &self.color_texture.as_ref().ok_or(wgpu::SurfaceError::Lost)?.view,
                    depth_slice: None,
                    resolve_target: Some(&self.scene_color_texture.as_ref().ok_or(wgpu::SurfaceError::Lost)?.view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), 
                        store: wgpu::StoreOp::Discard,
                    },
                }
        };


        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Total Scene Pass"),
            color_attachments: &[Some(render_pass_color_attachments)],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_stencil.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load, // <- clear depth again
                    store: wgpu::StoreOp::Store,
                }),

                stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load, // <- keep mask
                //load: wgpu::LoadOp::Clear(0), 
                store: wgpu::StoreOp::Store,
                }) ,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
           
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_stencil_reference(1);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(2, &self.spin_bind_group, &[]);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
   
        render_pass.draw_mesh_instanced(&self.obj_model.meshes[0], 0..self.instances.len() as u32);

        drop(render_pass);



        // /  C O M P O S I T E   P A S S 

        let mut composite_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("composite pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view, // ⬅ swapchain
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        composite_pass.set_pipeline(&self.composite_pipeline);
        composite_pass.set_bind_group(0, &self.composite_bind_group, &[]);
        composite_pass.draw(0..3, 0..1);

        drop(composite_pass);



        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Space, is_pressed) =>  {
                            if is_pressed
                            {self.is_paused = !self.is_paused;
                            self.last_frame = Instant::now();}
                        }
            (
                KeyCode::KeyW
                | KeyCode::ArrowUp
                | KeyCode::KeyA
                | KeyCode::ArrowLeft
                | KeyCode::KeyS
                | KeyCode::ArrowDown
                | KeyCode::KeyD
                | KeyCode::ArrowRight,
                is_pressed,
            ) => self.camera_controller.handle_key(code, is_pressed),
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}