pub mod camera;
// pub mod geometry;
pub mod shader;
pub mod ssbo;
pub mod transform;
pub mod types;
// use crate::geometry::Geometry;

pub use camera::Camera;
pub use transform::Transform;

use crate::{
    shader::{BindGroupBuilder, BindGroupLayoutBuilder, BindGroupPool, ShaderBuilder},
    ssbo::SsboPool,
};

pub struct Scene {
    nodes: Vec<Node>,
}

pub struct Node {
    transform: Transform,
}

#[derive(Debug, Clone)]
pub struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Context {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        });
        let adapter = pollster::block_on(adapter).unwrap();

        let device_queue = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features {
                features_wgpu: wgpu::FeaturesWGPU::PUSH_CONSTANTS,
                features_webgpu: wgpu::FeaturesWebGPU::empty(),
            },
            required_limits: wgpu::Limits {
                max_push_constant_size: 128,
                ..Default::default()
            },
            ..Default::default()
        });
        let (device, queue) = pollster::block_on(device_queue).unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

#[derive(Debug, Clone)]
pub struct Renderer {
    ctx: Context,
    bind_group_pool: BindGroupPool,
    ssbo_pool: SsboPool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            ctx: Context::new(),
            bind_group_pool: BindGroupPool::new(),
            ssbo_pool: SsboPool::new(),
        }
    }

    pub fn shader(&self) -> ShaderBuilder {
        ShaderBuilder::new(&self.ctx)
    }

    pub fn bindgroup_layout(&self) -> BindGroupLayoutBuilder {
        BindGroupLayoutBuilder::new(&self.ctx)
    }

    pub fn bindgroup(&self, layout: wgpu::BindGroupLayout) -> BindGroupBuilder<'_> {
        BindGroupBuilder::new(&self, layout)
    }
}

struct RenderTarget {
    // ...
}

struct Obj {
    bindgroups: smallvec::SmallVec<[wgpu::BindGroup; 2]>,
    shader: shader::Shader,
    geometry: (),
    push_constants: (),
}