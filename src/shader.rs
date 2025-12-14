// pub mod modules;

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
    source: SmallVec<[ShaderSource; 1]>,
    vertex_entry: Box<str>, // TODO: replace with small_str
    fragment_entry: Box<str>, // TODO: replace with small_str
    shader_module: wgpu::ShaderModule,
    layout: (),
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
    fn object_ssbo<T>() {}
    // should accept push constant description
    fn object_ubo<T>() {}
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
