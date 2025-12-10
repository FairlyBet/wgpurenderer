use nohash_hasher::{BuildNoHashHasher, IsEnabled};
use sorted_vec::SortedVec;
use std::{any::TypeId, cell::Cell, cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

type TypeIdHashMap<V> = HashMap<TypeIdKey, V, BuildNoHashHasher<TypeIdKey>>;

type StagingBuffer = Vec<u8>;

type InstanceCounter = Rc<Cell<u32>>;

pub trait UniformData: bytemuck::NoUninit {}

impl<T> UniformData for T where T: bytemuck::NoUninit {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniformId(u32);

/// Newtype wrapper around TypeId for use with nohash-hasher.
/// TypeId is internally a u64, so we can use it directly as a hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TypeIdKey(TypeId);

impl TypeIdKey {
    pub fn new<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

impl IsEnabled for TypeIdKey {}

impl From<TypeId> for TypeIdKey {
    fn from(id: TypeId) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
struct UniformRegistryInner {
    uniforms: TypeIdHashMap<UniformType>,
    global_uniforms: TypeIdHashMap<GlobalUniformType>,
}

impl UniformRegistryInner {
    fn get_uniform_type<T: bytemuck::NoUninit>(&mut self) -> &mut UniformType {
        let key = TypeIdKey::new::<T>();

        self.uniforms
            .entry(key)
            .or_insert_with(|| UniformType::new::<T>())
    }

    fn upload_uniform<T: bytemuck::NoUninit>(&mut self, uniform: &Uniform, val: &T) {
        let entry = self.get_uniform_type::<T>();

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

#[derive(Debug, Clone)]
pub struct UniformRegistry {
    inner: Rc<RefCell<UniformRegistryInner>>,
}

impl UniformRegistry {
    fn upload_uniform<T: bytemuck::NoUninit>(&self, uniform: &Uniform, val: &T) {
        self.inner.borrow_mut().upload_uniform(uniform, val);
    }

    fn remove_uniform(&mut self, id: UniformId) {
        self.inner.borrow_mut().remove_uniform(id);
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

#[derive(Debug)]
struct UniformType {
    size: usize,
    align: usize,
    type_id: TypeId,
    name: &'static str,
    staging_buffer: StagingBuffer,
    uniforms: SortedVec<UniformEntry>,
}

impl UniformType {
    fn new<T: 'static>() -> Self {
        Self {
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            type_id: TypeId::of::<T>(),
            name: std::any::type_name::<T>(),
            staging_buffer: vec![],
            uniforms: SortedVec::new(),
        }
    }

    fn find_by_id(&self, id: UniformId) -> Result<usize, usize> {
        self.uniforms.binary_search_by_key(&id, |entry| entry.id)
    }
}

#[derive(Debug)]
struct GlobalUniformType {
    uniforms: Vec<UniformId>,
    staging_buffer: StagingBuffer,
}

impl GlobalUniformType {
    fn upload<T>(&mut self) {
        //
    }
}

#[derive(Debug)]
pub struct Uniform {
    id: UniformId,
    uniform_register: UniformRegistry,
    counter: InstanceCounter,
}

impl Uniform {
    pub fn upload<T: bytemuck::NoUninit>(&self, val: &T) {
        self.uniform_register.upload_uniform(self, val);
    }

    pub fn add_global_uniform<T: bytemuck::NoUninit>() {
        //
    }
}

impl Clone for Uniform {
    fn clone(&self) -> Self {
        self.counter.update(|v| v + 1);

        Self {
            id: self.id,
            uniform_register: self.uniform_register.clone(),
            counter: self.counter.clone(),
        }
    }
}

impl Drop for Uniform {
    fn drop(&mut self) {
        self.counter.update(|v| v - 1);

        if self.counter.get() == 0 {
            self.uniform_register.remove_uniform(self.id);
        }
    }
}
