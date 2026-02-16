pub mod camera;
// pub mod geometry;
pub mod shader;
// pub mod ssbo;
pub mod transform;
pub mod types;
// use crate::geometry::Geometry;

pub use camera::Camera;
pub use transform::Transform;
use wgpu::{DeviceDescriptor, ExperimentalFeatures, Features, Limits};

use crate::shader::{
    BindGroupBuilder, BindGroupLayoutBuilder, Managed, ManagedEntry, ShaderBuilder,
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

        let device_queue = adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::IMMEDIATES,
            required_limits: Limits::defaults(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
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
    // bind_group_pool: BindGroupPool,
    // ssbo_pool: SsboPool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            ctx: Context::new(),
            // bind_group_pool: BindGroupPool::new(),
            // ssbo_pool: SsboPool::new(),
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

pub struct DrawCall {
    geometry: Geometry,
    shader: Shader,
    params: Params,
}

#[derive(Debug, Clone)]
pub struct Geometry {
    index_buffer: Option<wgpu::Buffer>,
    buffers: Vec<wgpu::Buffer>,
}

#[derive(Debug, Clone)]
pub struct Shader {
    shader_module: wgpu::ShaderModule,
    bind_groupes: Vec<wgpu::BindGroup>,
    immediates: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct Params {
    blend: Option<wgpu::BlendState>,
    write_mask: wgpu::ColorWrites,
    cull_mode: Option<wgpu::Face>,
    polygon_mode: wgpu::PolygonMode,
    unclipped_depth: bool,
    depth_format: wgpu::TextureFormat,
    depth_write_enabled: bool,
    depth_compare: wgpu::CompareFunction,
    multisample_count: u32,
    multisample_mask: u64,
    alpha_to_coverage_enabled: bool,
}
