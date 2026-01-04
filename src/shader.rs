// pub mod modules;

use crate::{Context, Renderer, types};
use smallvec::SmallVec;
use sorted_vec::SortedVec;
use std::{borrow::Cow, cell::RefCell, fmt::Debug, ops::Range, rc::Rc};

pub struct ShaderBuilder {
    bind_group_layouts: SmallVec<[wgpu::BindGroupLayout; 2]>,
    vertex_entry: Cow<'static, str>,
    fragment_entry: Cow<'static, str>,
    source: SmallVec<[Cow<'static, str>; 1]>,
    ctx: Context,
}

impl ShaderBuilder {
    pub(crate) fn new(ctx: &Context) -> Self {
        Self {
            bind_group_layouts: smallvec::smallvec![],
            vertex_entry: "".into(),
            fragment_entry: "".into(),
            source: smallvec::smallvec![],
            ctx: ctx.clone(),
        }
    }

    pub fn bind_group_layout(mut self, value: wgpu::BindGroupLayout) -> Self {
        self.bind_group_layouts.push(value);
        self
    }

    pub fn source(mut self, source: impl Into<Cow<'static, str>>) -> Self {
        self.source.push(source.into());
        self
    }

    pub fn entries(
        mut self,
        vertex_entry: impl Into<Cow<'static, str>>,
        fragment_entry: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.vertex_entry = vertex_entry.into();
        self.fragment_entry = fragment_entry.into();
        self
    }

    pub fn build(self) -> Shader {
        let source: String = self.source.iter().map(|v| v.chars()).flatten().collect();

        let shader_module = self
            .ctx
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        Shader {
            shader_module,
            bind_group_layouts: self.bind_group_layouts,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shader {
    shader_module: wgpu::ShaderModule,
    bind_group_layouts: SmallVec<[wgpu::BindGroupLayout; 2]>,
}

impl Shader {
    pub fn bind_group_layout(&self, n: usize) -> &wgpu::BindGroupLayout {
        &self.bind_group_layouts[n]
    }
}

pub trait BindingResource {
    fn as_binding(&self) -> wgpu::BindingResource<'_>;
}

impl BindingResource for wgpu::Buffer {
    fn as_binding(&self) -> wgpu::BindingResource<'_> {
        self.as_entire_binding()
    }
}

pub struct BindGroupLayoutBuilder {
    entries: SmallVec<[wgpu::BindGroupLayoutEntry; 3]>,
    ctx: Context,
}

impl BindGroupLayoutBuilder {
    pub(crate) fn new(ctx: &Context) -> Self {
        Self {
            entries: smallvec::smallvec![],
            ctx: ctx.clone(),
        }
    }

    pub fn entry(mut self, entry: wgpu::BindGroupLayoutEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn build(self) -> wgpu::BindGroupLayout {
        let bindgroup_layout =
            self.ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &self.entries,
                });
        bindgroup_layout
    }
}

pub struct BindGroupBuilder<'a> {
    resources: SmallVec<[wgpu::BindGroupEntry<'a>; 3]>,
    layout: wgpu::BindGroupLayout,
    renderer: Renderer,
}

impl<'a> BindGroupBuilder<'a> {
    pub(crate) fn new(renderer: &Renderer, layout: wgpu::BindGroupLayout) -> Self {
        Self {
            resources: smallvec::smallvec![],
            layout,
            renderer: renderer.clone(),
        }
    }

    pub fn entry(mut self, binding: u32, resource: &'a dyn BindingResource) -> Self {
        self.resources.push(wgpu::BindGroupEntry {
            binding,
            resource: resource.as_binding(),
        });
        self
    }

    pub fn build(self) -> wgpu::BindGroup {
        let bind_group = self
            .renderer
            .ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.layout,
                entries: &self.resources,
            });
        bind_group
    }
}

// pub struct CountedBindGroup {
//     counter: types::InstanceCounter,
//     id: types::InstanceId,
//     bind_group: wgpu::BindGroup,
// }

// impl Clone for CountedBindGroup {
//     fn clone(&self) -> Self {
//         self.counter.update(|v| v + 1);
//         Self {
//             counter: self.counter.clone(),
//             id: self.id.clone(),
//             bind_group: self.bind_group.clone(),
//         }
//     }
// }

// impl Drop for CountedBindGroup {
//     fn drop(&mut self) {
//         self.counter.update(|v| v - 1);
//         if self.counter.get() == 0 {
//             // release id
//         }
//     }
// }

// #[derive(Debug, Default)]
// pub struct BindGroupPoolInner {
//     id_pool: types::IdPool,
// }

// impl BindGroupPoolInner {
//     pub fn bind_group(&mut self, bind_group: wgpu::BindGroup) -> CountedBindGroup {
//         CountedBindGroup {
//             counter: types::InstanceCounter::new(Cell::new(1)),
//             id: self.id_pool.get_next(),
//             bind_group,
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct BindGroupPool {
//     inner: Rc<RefCell<BindGroupPoolInner>>,
// }

// impl BindGroupPool {
//     pub fn new() -> Self {
//         Self {
//             inner: Rc::new(RefCell::new(BindGroupPoolInner::default())),
//         }
//     }
// }

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
struct CollectionInner<T> {
    data: SortedVec<Entry<T>>,
    id_pool: types::IdPool,
}

impl<T> CollectionInner<T> {
    fn create(&mut self, val: T) -> types::InstanceId {
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

impl<T: Clone> CollectionInner<T> {
    fn get_clone(&self, id: types::InstanceId) -> T {
        let index = self
            .data
            .binary_search_by_key(&id, |item| item.id)
            .expect("Value is not present");
        self.data[index].val.clone()
    }
}

#[derive(Debug)]
struct Collection<T> {
    inner: Rc<RefCell<CollectionInner<T>>>,
}

impl<T> Collection<T> {
    fn create(&self, val: T) -> types::InstanceId {
        self.inner.borrow_mut().create(val)
    }

    fn delete(&self, id: types::InstanceId) {
        self.inner.borrow_mut().delete(id);
    }
}

impl<T: Clone> Collection<T> {
    fn get_clone(&self, id: types::InstanceId) -> T {
        self.inner.borrow().get_clone(id)
    }
}

impl<T> Clone for Collection<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct Managed<T> {
    instance_counter: types::InstanceCounter,
    instance_id: types::InstanceId,
    collection: Collection<T>,
}

impl<T> Clone for Managed<T> {
    fn clone(&self) -> Self {
        self.instance_counter.increment();
        Self {
            instance_counter: self.instance_counter.clone(),
            instance_id: self.instance_id.clone(),
            collection: self.collection.clone(),
        }
    }
}

impl<T> Drop for Managed<T> {
    fn drop(&mut self) {
        self.instance_counter.decrement();
        if self.instance_counter.value() == 0 {
            self.collection.delete(self.instance_id);
        }
    }
}

struct PushConstantBuffer {
    buffer: Vec<u8>,
}

struct PushConstant(Range<usize>);

fn foo(p: Managed<PushConstant>) {
    
}
