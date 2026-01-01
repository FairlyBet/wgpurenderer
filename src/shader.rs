// pub mod modules;

use crate::{Context, Renderer, types};
use smallvec::{SmallVec, smallvec};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
};

pub struct ShaderBuilder {
    bindgroup_layouts: Vec<wgpu::BindGroupLayout>,
    vertex_entry: Cow<'static, str>,
    fragment_entry: Cow<'static, str>,
    source: SmallVec<[Cow<'static, str>; 1]>,
    ctx: Context,
}

impl ShaderBuilder {
    pub(crate) fn new(ctx: &Context) -> Self {
        Self {
            bindgroup_layouts: vec![],
            vertex_entry: "".into(),
            fragment_entry: "".into(),
            source: smallvec![],
            ctx: ctx.clone(),
        }
    }

    pub fn bindgroup(mut self, value: wgpu::BindGroupLayout) -> Self {
        self.bindgroup_layouts.push(value);
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
            bindgroup: self.bindgroup_layouts,
        }
    }
}

pub struct Shader {
    shader_module: wgpu::ShaderModule,
    bindgroup: Vec<wgpu::BindGroupLayout>,
}

// TODO: make sealed
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

pub struct CountedBindGroup {
    counter: types::InstanceCounter,
    id: types::InstanceId,
    bind_group: wgpu::BindGroup,
}

impl Clone for CountedBindGroup {
    fn clone(&self) -> Self {
        self.counter.update(|v| v + 1);
        Self {
            counter: self.counter.clone(),
            id: self.id.clone(),
            bind_group: self.bind_group.clone(),
        }
    }
}

impl Drop for CountedBindGroup {
    fn drop(&mut self) {
        self.counter.update(|v| v - 1);
        if self.counter.get() == 0 {
            // release id
        }
    }
}

#[derive(Debug, Default)]
pub struct BindGroupPoolInner {
    id_pool: types::IdPool,
}

impl BindGroupPoolInner {
    pub fn bind_group(&mut self, bind_group: wgpu::BindGroup) -> CountedBindGroup {
        CountedBindGroup {
            counter: types::InstanceCounter::new(Cell::new(1)),
            id: self.id_pool.get_next(),
            bind_group,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindGroupPool {
    inner: Rc<RefCell<BindGroupPoolInner>>,
}
impl BindGroupPool {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(BindGroupPoolInner::default())),
        }
    }
}
