// pub mod modules;

use crate::RenderContext;
use smallvec::{SmallVec, smallvec};
use std::borrow::Cow;

pub struct ShaderBuilder {
    bindgroup: Vec<wgpu::BindGroupLayoutEntry>,
    vertex_entry: Cow<'static, str>,
    fragment_entry: Cow<'static, str>,
    source: SmallVec<[Cow<'static, str>; 1]>,
    ctx: RenderContext,
}

impl ShaderBuilder {
    pub(crate) fn new(ctx: &RenderContext) -> Self {
        Self {
            bindgroup: vec![],
            vertex_entry: "".into(),
            fragment_entry: "".into(),
            source: smallvec![],
            ctx: ctx.clone(),
        }
    }

    pub fn bindgroup(mut self, entry: wgpu::BindGroupLayoutEntry) -> Self {
        self.bindgroup.push(entry);
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
            bindgroup: self.bindgroup,
        }
    }
}

pub struct Shader {
    shader_module: wgpu::ShaderModule,
    bindgroup: Vec<wgpu::BindGroupLayoutEntry>,
}

// TODO: make sealed
pub trait Binding {
    fn as_binding(&self) -> wgpu::BindingResource<'_>;
}
