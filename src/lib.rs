pub mod camera;
pub mod transform;
pub mod utils;

use crate::utils::InstanceCounter;
pub use camera::Camera;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use sorted_vec::SortedVec;
use std::{borrow::Cow, cell::RefCell, num::NonZeroU32, ops::Range, rc::Rc};
use transform::Transform;
use wgpu::{DeviceDescriptor, ExperimentalFeatures, Features, Limits};

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

#[derive(Debug, Clone)]
pub struct Renderer {
    ctx: Context,
    bind_group_storage: Storage<wgpu::BindGroup>,
    render_pipeline_storage: Storage<wgpu::RenderPipeline>,
    shader_cache: FxHashMap<Box<str>, wgpu::ShaderModule>,
    bind_group_layout_cache: Vec<(Vec<wgpu::BindGroupLayoutEntry>, wgpu::BindGroupLayout)>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            ctx: Context::new(),
            bind_group_storage: Storage::new(),
            render_pipeline_storage: Storage::new(),
            shader_cache: FxHashMap::default(),
            bind_group_layout_cache: Vec::new(),
        }
    }

    pub fn create_material(&mut self, material: &Material) -> Handle<wgpu::RenderPipeline> {
        for item in &material.bind_groups {
            let found = self.bind_group_layout_cache.iter().find(|(entries, _)| *entries == *item);

            if found.is_none() {
                let bind_group_layout =
                    self.ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: item.as_slice(),
                    });
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
        let layout = self.ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            immediate_size: 128,
        });

        let shader_module = self.shader_cache.entry(material.source.clone()).or_insert_with(|| {
            self.ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&material.source)),
            })
        });

        let vertex = wgpu::VertexState {
            module: shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &material.veretex.buffers,
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

        let render_pipeline = self.ctx.device.create_render_pipeline(&desc);
        let id = self.render_pipeline_storage.create(render_pipeline);

        Handle {
            id,
            counter: InstanceCounter::new(),
            storage: self.render_pipeline_storage.clone(),
        }
    }

    pub fn create_bindgroup(
        &mut self,
        layout_entries: &[wgpu::BindGroupLayoutEntry],
        entries: &[wgpu::BindGroupEntry],
    ) -> Handle<wgpu::BindGroup> {
        let found = self
            .bind_group_layout_cache
            .iter()
            .position(|(cached_entries, _)| cached_entries.as_slice() == layout_entries);

        let layout = if let Some(index) = found {
            &self.bind_group_layout_cache[index].1
        } else {
            let bind_group_layout =
                self.ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: layout_entries,
                });
            self.bind_group_layout_cache.push((layout_entries.to_vec(), bind_group_layout));
            &self.bind_group_layout_cache.last().unwrap().1
        };

        let bind_group = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries,
        });

        let id = self.bind_group_storage.create(bind_group);

        Handle {
            id,
            counter: InstanceCounter::new(),
            storage: self.bind_group_storage.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawCall {
    geometry: Geometry,
    shader_data: ShaderData,
    instance_count: NonZeroU32,
    render_pipeline_handle: Handle<wgpu::RenderPipeline>,
}

#[derive(Debug, Clone)]
pub struct Geometry {
    index_buffer: Option<wgpu::Buffer>,
    buffers: Vec<(wgpu::Buffer, Option<Range<u64>>)>,
    vertex_count: u32,
    index_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    bind_groups: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    veretex: Vertex,
    fragment: Option<Fragment>,
    depth_stencil: Option<wgpu::DepthStencilState>,
    primitive: wgpu::PrimitiveState,
    multisample: wgpu::MultisampleState,
    source: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vertex {
    buffers: Vec<wgpu::VertexBufferLayout<'static>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    targets: Vec<Option<wgpu::ColorTargetState>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthStencil {
    depth_format: wgpu::TextureFormat,
    depth_write_enabled: bool,
    depth_compare: wgpu::CompareFunction,
    stencil: wgpu::StencilState,
    unclipped_depth: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Primitive {
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    front_face: wgpu::FrontFace,
    cull_mode: Option<wgpu::Face>,
}

#[derive(Debug, Clone)]
pub struct ShaderData {
    immediates: Vec<u8>,
    bind_groups: SmallVec<[Handle<wgpu::BindGroup>; 3]>,
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

#[derive(Debug)]
struct StorageInner<T> {
    data: SortedVec<Entry<T>>,
    id_pool: utils::IdPool,
}

impl<T> StorageInner<T> {
    fn new() -> Self {
        Self {
            data: SortedVec::new(),
            id_pool: utils::IdPool::new(),
        }
    }

    fn insert(&mut self, val: T) -> utils::InstanceId {
        let id = self.id_pool.get_next();
        let entry = Entry {
            val,
            id,
        };
        self.data.push(entry);
        id
    }

    fn delete(&mut self, id: utils::InstanceId) {
        self.id_pool.free(id);
        let index =
            self.data.binary_search_by_key(&id, |item| item.id).expect("Value is not present");
        _ = self.data.remove_index(index);
    }
}

#[derive(Debug)]
struct Storage<T> {
    inner: Rc<RefCell<StorageInner<T>>>,
}

impl<T> Storage<T> {
    fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(StorageInner::new())),
        }
    }

    fn create(&self, val: T) -> utils::InstanceId {
        self.inner.borrow_mut().insert(val)
    }

    fn delete(&self, id: utils::InstanceId) {
        self.inner.borrow_mut().delete(id);
    }
}

impl<T> Clone for Storage<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Handle<T> {
    id: utils::InstanceId,
    counter: utils::InstanceCounter,
    storage: Storage<T>,
}

impl Handle<wgpu::RenderPipeline> {
    pub fn get_bind_group_layout(&self, index: u32) -> wgpu::BindGroupLayout {
        let storage = self.storage.inner.borrow();
        let item = storage.data.binary_search_by(|item| item.id.cmp(&self.id)).unwrap();
        storage.data[item].val.get_bind_group_layout(index)
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        self.counter.increment();
        Self {
            id: self.id,
            counter: self.counter.clone(),
            storage: self.storage.clone(),
        }
    }
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        self.counter.decrement();
        if self.counter.value() == 0 {
            self.storage.delete(self.id);
        }
    }
}
