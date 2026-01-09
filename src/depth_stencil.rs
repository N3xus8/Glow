pub struct StencilTexture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl StencilTexture {
    pub fn create_stencil_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
        sample_count: u32,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let depth_stencil_format = wgpu::TextureFormat::Depth24PlusStencil8;

        let depth_stencil_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: depth_stencil_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_stencil_view = depth_stencil_texture.create_view(&Default::default());

        Self {
            texture: depth_stencil_texture,
            view: depth_stencil_view,
        }
    }

    pub fn create_depth_only_view(&self, label: &str) -> wgpu::TextureView {

        self.texture.create_view(
            &wgpu::TextureViewDescriptor {
            label: Some(label),
                format: None, // Inherit from texture
                dimension: None, // Inherit from texture
                // These fields are now direct members of TextureViewDescriptor:
                aspect: wgpu::TextureAspect::DepthOnly, 
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
                usage: None,
            })
    }
}
