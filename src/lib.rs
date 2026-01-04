pub mod app;
pub use app::App;
pub mod camera;
pub mod depth_stencil;
pub mod extra;
pub mod model;
pub mod pipeline;
pub mod resources;
pub mod state;
pub mod texture;
pub mod utils;
pub mod vertex;
pub mod web_utils;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);
