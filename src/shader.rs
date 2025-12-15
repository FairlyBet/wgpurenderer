// pub mod modules;

use crate::{RenderContext, types::*};
use smallvec::SmallVec;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderSource(Cow<'static, str>);

impl ShaderSource {
    pub fn src(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for ShaderSource {
    fn from(value: &'static str) -> Self {
        Self(value.into())
    }
}

impl From<String> for ShaderSource {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

pub struct ShaderTemplate {
    shader_module: wgpu::ShaderModule,
    per_frame_bindgroup: wgpu::BindGroup,
    // internal bindgroup
    // dynamic bindgroup
}

impl ShaderTemplate {
    pub fn create_dynamic_bindgroup() {}

    // Builder methods
    // add description for these
    fn dynamic_ssbo() {}
    fn dynamic_ubo() {}
    fn dynamic_texture() {}
    fn dynamic_push_constants() {}

    // value
    fn ssbo() {}
    // value
    fn ubo() {}
    // value
    fn texture() {}
    // should accept push constant description
    fn object_data<T: bytemuck::NoUninit>() {}
    // should accept push constant description
    fn uniform_data<T: bytemuck::NoUninit>() {}
}

pub struct ShaderBuilder<'a> {
    // TODO: check for duplicates
    object_data: Vec<(TypeId, u32)>,
    uniform_data: Vec<(TypeId, u32)>,
    binding_resources: Vec<(&'a dyn Binding, u32)>,
    vertex_entry: Box<str>,   // TODO: replace with small_str
    fragment_entry: Box<str>, // TODO: replace with small_str
    source: SmallVec<[ShaderSource; 1]>,
    ctx: RenderContext,
}

impl<'a> ShaderBuilder<'a> {
    pub fn object_data<T: bytemuck::NoUninit>(
        mut self,
        binding: u32,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        self.object_data.push((TypeId::new::<T>(), binding));
        self
    }

    pub fn uniform_data<T: bytemuck::NoUninit>(
        mut self,
        binding: u32,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        self.uniform_data.push((TypeId::new::<T>(), binding));
        self
    }

    pub fn binding_resource(
        mut self,
        binding_resource: &'a dyn Binding,
        binding: u32,
        visibility: wgpu::ShaderStages,
        r#type: wgpu::BindingType,
    ) -> Self {
        self.binding_resources.push((binding_resource, binding));
        self
    }

    pub fn source(mut self, source: ShaderSource) -> Self {
        self.source.push(source);
        self
    }

    pub fn entries(mut self, vertex_entry: &str, fragment_entry: &str) -> Self {
        self.vertex_entry = vertex_entry.into();
        self.fragment_entry = fragment_entry.into();
        self
    }

    pub fn build(self) -> ShaderTemplate {
        let layout = self
            .ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[],
            });
        // create bind group for 0 group
        todo!()
    }
}

pub struct Shader {
    // rc for Shader Template
    // id
    //
}

impl Shader {
    // Provide actual values
    // Also during build
    fn dynamic_ssbo() {}
    fn dynamic_ubo() {}
    fn dynamic_texture() {}
    fn dynamic_push_constants() {}
}

// TODO: make sealed
pub trait Binding {
    fn as_binding(&self) -> wgpu::BindingResource<'_>;
}
