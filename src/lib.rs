pub mod camera;
pub mod geometry;
pub mod shader;
pub mod transform;

use std::{
    any::TypeId,
    cell::{Cell, RefCell},
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::Rc,
};

pub use camera::Camera;
use nohash_hasher::{BuildNoHashHasher, IsEnabled};
use sorted_vec::SortedVec;
pub use transform::Transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniformId(u32);

/// Newtype wrapper around TypeId for use with nohash-hasher.
/// TypeId is internally a u64, so we can use it directly as a hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeIdKey(TypeId);

impl Hash for TypeIdKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TypeId's Hash impl writes a u64, which is what nohash-hasher expects
        self.0.hash(state);
    }
}

impl IsEnabled for TypeIdKey {}

impl From<TypeId> for TypeIdKey {
    fn from(id: TypeId) -> Self {
        Self(id)
    }
}

type TypeIdHashMap<V> = HashMap<TypeIdKey, V, BuildNoHashHasher<TypeIdKey>>;

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
    uniforms: TypeIdHashMap<UniformData>,
    global_uniforms: TypeIdHashMap<GlobalUniformData>,
}

impl UniformRegister {
    fn get_uniform_data<T: bytemuck::NoUninit>(&mut self) -> &mut UniformData {
        let key = TypeIdKey::from(TypeId::of::<T>());
        self.uniforms
            .entry(key)
            .or_insert_with(|| UniformData::new::<T>())
    }

    pub fn upload_uniform<T: bytemuck::NoUninit>(&mut self, uniform: &Uniform, val: &T) {
        let entry = self.get_uniform_data::<T>();
        let slice = bytemuck::bytes_of(val);
        match entry.find_by_id(uniform.id) {
            Ok(index) => {
                let offset = entry.uniforms[index].offset;
                entry.staging_buffer[offset..offset + entry.size].copy_from_slice(slice);
            }
            Err(_) => {
                let offset = entry.staging_buffer.len();
                entry.staging_buffer.extend_from_slice(slice);
                entry.uniforms.insert(UniformEntry {
                    id: uniform.id,
                    offset,
                });
            }
        }
    }

    fn remove_uniform(&mut self, id: UniformId) {
        // TODO: free buffers
        for data in self.uniforms.values_mut() {
            if let Ok(index) = data.find_by_id(id) {
                data.uniforms.remove_index(index);
                return;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UniformEntry {
    id: UniformId,
    offset: usize,
}

impl PartialOrd for UniformEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UniformEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

struct UniformData {
    size: usize,
    align: usize,
    type_id: TypeId,
    staging_buffer: Vec<u8>,
    uniforms: SortedVec<UniformEntry>,
}

impl UniformData {
    pub fn new<T: 'static>() -> Self {
        Self {
            size: size_of::<T>(),
            align: align_of::<T>(),
            type_id: TypeId::of::<T>(),
            staging_buffer: vec![],
            uniforms: SortedVec::new(),
        }
    }

    fn find_by_id(&self, id: UniformId) -> Result<usize, usize> {
        self.uniforms.binary_search_by_key(&id, |entry| entry.id)
    }
}

struct GlobalUniformData {
    uniforms: Vec<UniformId>,
    buffer: Vec<u8>
}

pub struct Uniform {
    id: UniformId,
    uniform_register: Rc<RefCell<UniformRegister>>,
    counter: Rc<Cell<usize>>,
}

impl Uniform {
    pub fn upload<T: bytemuck::NoUninit>(&self, data: T) {
        //
    }

    pub fn add_global_uniform<T: bytemuck::NoUninit>() {
        //
    }
}

impl Drop for Uniform {
    fn drop(&mut self) {
        let mut counter = self.counter.get();
        counter -= 1;
        if counter == 0 {
            self.uniform_register.borrow_mut().remove_uniform(self.id);
        }
    }
}
