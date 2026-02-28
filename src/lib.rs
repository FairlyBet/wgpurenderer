pub mod camera;
pub mod renderpass;
pub mod transform;
pub mod utils;

pub use camera::Camera;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use sorted_vec::SortedVec;
use std::{
    borrow::Cow, cell::RefCell, fmt::Debug, marker::PhantomData, num::NonZeroU32, ops::Range,
    rc::Rc,
};
use transform::Transform;
use wgpu::{DeviceDescriptor, ExperimentalFeatures, Features, Limits};

use crate::{renderpass::RenderPass, utils::TypeId};

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
            backends: wgpu::Backends::VULKAN,
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
            required_limits: Limits {
                max_immediate_size: 128,
                ..Default::default()
            },
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

#[derive(Debug)]
pub struct Renderer {
    context: Context,
    shader_cache: FxHashMap<Cow<'static, str>, wgpu::ShaderModule>,
    bind_group_layout_cache: Vec<(Vec<wgpu::BindGroupLayoutEntry>, wgpu::BindGroupLayout)>,
    pub surface: Option<wgpu::Surface<'static>>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub surface_view: Option<wgpu::TextureView>,
    // TODO: determine duplicating render pipelines and return the existing one
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            shader_cache: FxHashMap::default(),
            bind_group_layout_cache: Vec::new(),
            surface: None,
            config: None,
            surface_texture: None,
            surface_view: None,
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn init_surface(&mut self, surface: wgpu::Surface<'static>, width: u32, height: u32) {
        let surface_caps = surface.get_capabilities(self.context.adapter());
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(self.context.device(), &config);

        self.surface = Some(surface);
        self.config = Some(config);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let (Some(surface), Some(config)) = (&mut self.surface, &mut self.config) {
                config.width = width;
                config.height = height;
                surface.configure(self.context.device(), config);
            }
        }
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.as_ref().map(|c| c.format).unwrap_or(wgpu::TextureFormat::Bgra8Unorm)
    }

    pub fn acquire(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Some(surface) = &self.surface {
            let output = surface.get_current_texture()?;
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.surface_texture = Some(output);
            self.surface_view = Some(view);
        }
        Ok(())
    }

    pub fn present(&mut self) {
        self.surface_view = None;
        if let Some(texture) = self.surface_texture.take() {
            texture.present();
        }
    }

    pub fn create_render_pipeline(&mut self, material: &Material) -> wgpu::RenderPipeline {
        for item in &material.bind_groups {
            let found = self.bind_group_layout_cache.iter().find(|(entries, _)| *entries == *item);

            if found.is_none() {
                let bind_group_layout = self.context.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: item.as_slice(),
                    },
                );
                self.bind_group_layout_cache.push((item.clone(), bind_group_layout));
            }
        }

        let bind_group_layouts: Vec<_> = material
            .bind_groups
            .iter()
            .filter_map(|entries| self.bind_group_layout_cache.iter().find(|(e, _)| *entries == *e))
            .map(|i| &i.1)
            .collect();

        // TODO: add caching
        let layout = self.context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            immediate_size: 128,
        });

        let shader_module = self.shader_cache.entry(material.source.clone()).or_insert_with(|| {
            self.context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(material.source.clone()),
            })
        });

        let vertex = wgpu::VertexState {
            module: shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &material.vertex.buffers,
        };

        let fragment = material.fragment.as_ref().map(|f| wgpu::FragmentState {
            module: shader_module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &f.targets,
        });

        let desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex,
            primitive: material.primitive,
            depth_stencil: material.depth_stencil.clone(),
            multisample: material.multisample,
            fragment,
            multiview_mask: None,
            cache: None,
        };

        let render_pipeline = self.context.device.create_render_pipeline(&desc);
        render_pipeline
    }

    pub fn create_bindgroup(
        &mut self,
        layout_entries: &[wgpu::BindGroupLayoutEntry],
        entries: &[wgpu::BindGroupEntry],
    ) -> wgpu::BindGroup {
        let found = self
            .bind_group_layout_cache
            .iter()
            .position(|(cached_entries, _)| cached_entries.as_slice() == layout_entries);

        let layout = if let Some(index) = found {
            &self.bind_group_layout_cache[index].1
        } else {
            let bind_group_layout =
                self.context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: layout_entries,
                });
            self.bind_group_layout_cache.push((layout_entries.to_vec(), bind_group_layout));
            &self.bind_group_layout_cache.last().unwrap().1
        };

        let bind_group = self.context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries,
        });

        bind_group
    }

    pub fn render(&self, render_passes: &mut [&mut RenderPass]) {
        let mut encoder =
            self.context.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None,
            });

        for render_pass in render_passes.iter_mut() {
            render_pass.render(&mut encoder);
        }

        self.context.queue().submit(Some(encoder.finish()));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    pub bind_groups: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    pub vertex: Vertex,
    pub fragment: Option<Fragment>,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub primitive: wgpu::PrimitiveState,
    pub multisample: wgpu::MultisampleState,
    pub source: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vertex {
    pub buffers: Vec<wgpu::VertexBufferLayout<'static>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub targets: Vec<Option<wgpu::ColorTargetState>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthStencil {
    pub depth_format: wgpu::TextureFormat,
    pub depth_write_enabled: bool,
    pub depth_compare: wgpu::CompareFunction,
    pub stencil: wgpu::StencilState,
    pub unclipped_depth: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Primitive {
    pub topology: wgpu::PrimitiveTopology,
    pub polygon_mode: wgpu::PolygonMode,
    pub front_face: wgpu::FrontFace,
    pub cull_mode: Option<wgpu::Face>,
}

////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct DrawCall {
    pub geometry: Geometry,
    pub shader_data: ShaderData,
    pub instance_count: NonZeroU32,
    pub render_pipeline_handle: wgpu::RenderPipeline,
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_buffer_range: Option<Range<u32>>,
    pub index_format: wgpu::IndexFormat,
    pub buffers: Vec<(wgpu::Buffer, Option<Range<u32>>)>,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct ShaderData {
    pub immediates: Vec<u8>,
    pub bind_groups: SmallVec<[wgpu::BindGroup; 3]>,
}

#[derive(Debug, Clone)]
pub struct ColorAttachment {
    pub view: wgpu::TextureView,
    pub depth_slice: Option<u32>,
    pub resolve_target: Option<wgpu::TextureView>,
    pub ops: wgpu::Operations<wgpu::Color>,
}

#[derive(Debug, Clone)]
pub struct DepthStencilAttachment {
    pub view: wgpu::TextureView,
    pub depth_ops: Option<wgpu::Operations<f32>>,
    pub stencil_ops: Option<wgpu::Operations<u32>>,
}

#[derive(Debug, Clone)]
pub struct RenderTarget {
    pub color_attachments: SmallVec<[ColorAttachment; 1]>,
    pub depth_stencil_attachment: Option<DepthStencilAttachment>,
}

////////////////////////////////////////

#[derive(Debug)]
struct Entry<T> {
    val: T,
    id: utils::InstanceId,
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Entry<T> {}

impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/// Handles memory used for storing immeadiates
/// Returns handle with id which is used to index `entries` and find memory region that corresponds to the handle
#[derive(Debug)]
pub struct ImmediateManager {
    entries: Vec<Option<Range<usize>>>,
    id_pool: utils::IdPool,
    bytes: Vec<u8>,
}

impl ImmediateManager {
    pub fn new() -> Self {
        Self {
            id_pool: utils::IdPool::new(),
            entries: vec![],
            bytes: vec![],
        }
    }

    pub fn add(&mut self, size: usize) -> utils::InstanceId {
        let mut occupied: Vec<Range<usize>> = self.entries.iter().flatten().cloned().collect();
        occupied.sort_by_key(|r| r.start);

        let mut start_pos = 0;
        let mut found_range = None;

        for range in &occupied {
            if range.start - start_pos >= size {
                found_range = Some(start_pos..start_pos + size);
                break;
            }
            start_pos = range.end;
        }

        if found_range.is_none() {
            // No suitable gap found, check if there's enough space at the end.
            if self.bytes.len() - start_pos < size {
                // Grow the buffer with some spare space.
                let new_size = (self.bytes.len() * 2).max(start_pos + size * 2).max(64);
                self.bytes.resize(new_size, 0);
            }
            found_range = Some(start_pos..start_pos + size);
        }

        let range = found_range.unwrap();
        let id = self.id_pool.get_next();
        let idx = id.as_usize();

        if idx >= self.entries.len() {
            let new_len = (self.entries.len() * 2).max(idx + 1).max(8);
            self.entries.resize(new_len, None);
        }

        self.entries[idx] = Some(range);
        id
    }

    pub fn get(&self, id: utils::InstanceId) -> Option<&[u8]> {
        self.entries.get(id.as_usize())?.as_ref().map(|range| &self.bytes[range.clone()])
    }

    pub fn get_mut(&mut self, id: utils::InstanceId) -> Option<&mut [u8]> {
        self.entries.get(id.as_usize())?.as_ref().map(|range| &mut self.bytes[range.clone()])
    }

    pub fn remove(&mut self, id: utils::InstanceId) {
        if let Some(entry) = self.entries.get_mut(id.as_usize()) {
            if entry.take().is_some() {
                self.id_pool.free(id);
            }
        }
    }
}
