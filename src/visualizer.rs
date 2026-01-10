
use std::sync::Arc;

use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

pub struct Visualizer {
   pub surface: wgpu::Surface<'static>,
   pub device: wgpu::Device,
   pub queue: wgpu::Queue,
   pub config: wgpu::SurfaceConfiguration,


}

impl Visualizer {
    pub async fn new(window:  Arc<Window>) -> anyhow::Result<Self> {

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
                //alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            Ok(Self {surface, device, queue, config})

    }
}