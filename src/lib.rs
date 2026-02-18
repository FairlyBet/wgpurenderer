pub mod camera;
pub mod transform;
pub mod types;

pub use camera::Camera;
use rustc_hash::FxHashMap;
use sorted_vec::SortedVec;
use std::{cell::RefCell, num::NonZeroU32, ops::Range, rc::Rc};
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
    index_format: wgpu::IndexFormat,
}

#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    bind_group_layout: Vec<wgpu::BindGroupLayoutEntry>,
    veretex: Vertex,
    fragment: Vec<Fragment>,
    depth_stencil: DepthStencil,
    topology: Topology,
    multisample_count: u32,
    multisample_mask: u64,
    alpha_to_coverage_enabled: bool,
    source: Box<str>,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    buffers: Vec<wgpu::VertexBufferLayout<'static>>,
}

#[derive(Debug, Clone)]
pub struct Fragment {
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
    write_mask: wgpu::ColorWrites,
}

#[derive(Debug, Clone)]
pub struct DepthStencil {
    depth_format: wgpu::TextureFormat,
    depth_write_enabled: bool,
    depth_compare: wgpu::CompareFunction,
    stencil: wgpu::StencilState,
    unclipped_depth: bool,
}

#[derive(Debug, Clone)]
pub struct Topology {
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    front_face: wgpu::FrontFace,
    cull_mode: Option<wgpu::Face>,
}

#[derive(Debug, Clone)]
pub struct ShaderData {
    immediates: Vec<u8>,
    bind_groups: Vec<Handle<wgpu::BindGroup>>, // should have id
}

////////////////////////////////////////

#[derive(Debug)]
struct Entry<T> {
    val: T,
    id: types::InstanceId,
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
    id_pool: types::IdPool,
}

impl<T> StorageInner<T> {
    fn new() -> Self {
        Self {
            data: SortedVec::new(),
            id_pool: types::IdPool::new(),
        }
    }

    fn insert(&mut self, val: T) -> types::InstanceId {
        let id = self.id_pool.get_next();
        let entry = Entry { val, id };
        self.data.push(entry);
        id
    }

    fn delete(&mut self, id: types::InstanceId) {
        self.id_pool.free(id);
        let index = self
            .data
            .binary_search_by_key(&id, |item| item.id)
            .expect("Value is not present");
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

    fn create(&self, val: T) -> types::InstanceId {
        self.inner.borrow_mut().insert(val)
    }

    fn delete(&self, id: types::InstanceId) {
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
    id: types::InstanceId,
    counter: types::InstanceCounter,
    storage: Storage<T>,
}

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
