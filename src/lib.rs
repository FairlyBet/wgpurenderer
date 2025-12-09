pub mod camera;
pub mod geometry;
pub mod shader;
pub mod transform;

use std::any::TypeId;

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

struct UniformRegister {
    register: FastHashMap<TypeId, Vec<()>>,
}

impl UniformRegister {
    pub fn new_uniform<T: bytemuck::NoUninit>(&mut self) -> () {
        let t_name = std::any::type_name::<T>();
        let t_id = std::any::TypeId::of::<T>();
        let entry = self.register.entry(t_id);
        let val = entry.or_default();
        let size = size_of::<T>();
        let align = align_of::<T>();
    }
}

struct UniformData {
    size : usize,
    align: usize,
    type_id: TypeId,
}

struct Uniform< T > {
    data: T
}

impl<T> Uniform< T > {
    pub fn inner(&self) -> &T {
        &self.data
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

pub struct Object {
    id : u32
    // Arc Mutex Uniform register
}

impl Object {
    pub fn uniform_val< T >( &self, val : T ) {
        // writes val into uniform register
    }

    pub fn uniform< T >( &self ) {

    }
}
