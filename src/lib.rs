pub mod camera;
pub mod geometry;
pub mod shader;
pub mod transform;
pub mod uniform;

use crate::geometry::Geometry;

pub use camera::Camera;
pub use transform::Transform;
// pub use uniform::{Uniform, UniformData};

pub struct Scene {
    nodes: Vec<Node>,
}

pub struct Node {
    transform: Transform,
}

#[derive(Debug, Clone)]
pub struct RenderContext {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

pub struct Mesh {
    ctx: RenderContext,
    geometry: Geometry,
}

impl Mesh {
    // pub fn
}

pub struct ShaderSource {
    src: Vec<Box<str>>,
}

pub struct Renderer {}

impl Renderer {
    pub fn render(mesh: Mesh) {}
}

