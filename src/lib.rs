pub mod camera;
pub mod geometry;
pub mod shader;
pub mod transform;

use std::{any::TypeId, cell::RefCell, rc::Rc};

pub use camera::Camera;
pub use transform::Transform;
use wgpu::naga::FastHashMap;

use crate::geometry::Geometry;

pub struct Scene {
    nodes: Vec<Node>,
}

pub struct Node {
    transform: Transform,
}

#[derive(Debug, Clone)]
pub struct Context {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

pub struct Mesh {
    ctx: Context,
    geometry: Geometry,
}

impl Mesh {
    // pub fn
}

pub struct ShaderSource {
    src: Vec<Box<str>>,
}

pub struct Renderer {}

impl Renderer {
    pub fn render(mesh: Mesh) {}
}

pub struct UniformRegister {
    uniforms: FastHashMap<TypeId, UniformData>,
    global_uniforms: FastHashMap<TypeId, (Vec<u32>, Vec<u8>)>,
}

impl UniformRegister {
    fn get_uniform_data<T: bytemuck::NoUninit>(&mut self) -> &mut UniformData {
        let t_id = std::any::TypeId::of::<T>();
        self.uniforms.entry(t_id).or_insert(UniformData::new::<T>())
    }

    pub fn upload_uniform<T: bytemuck::NoUninit>(&mut self, uniform: Uniform, val: &T) {
        let entry = self.get_uniform_data::<T>();
        let pos = entry.uniforms.iter().position(|item| item.0 == uniform.id);
        let slice = bytemuck::bytes_of(val);
        match pos {
            Some(index) => {
                entry.staging_buffer[index..entry.size].copy_from_slice(slice);
            }
            None => {
                let pos = entry.staging_buffer.len();
                entry.staging_buffer.extend_from_slice(slice);
                entry.uniforms.push((uniform.id, pos));
            }
        }
    }

    fn remove_uniform(&mut self, id: u32) {
        //
    }
}

struct UniformData {
    size: usize,
    align: usize,
    type_id: TypeId,
    staging_buffer: Vec<u8>,
    // TODO: should be ordered vec (order by id)
    uniforms: Vec<(u32, usize)>,
}

impl UniformData {
    pub fn new<T: 'static>() -> Self {
        Self {
            size: size_of::<T>(),
            align: align_of::<T>(),
            type_id: TypeId::of::<T>(),
            staging_buffer: vec![],
            uniforms: vec![],
        }
    }
}

pub struct Uniform {
    id: u32,
    uniform_register: Rc<RefCell<UniformRegister>>,
}

impl Uniform {
    pub fn upload<T: bytemuck::NoUninit>(&self, data: T) {
        //
    }
}

impl Drop for Uniform {
    fn drop(&mut self) {
        self.uniform_register.borrow_mut().remove_uniform(self.id);
    }
}
