use crate::types::*;
use sorted_vec::SortedVec;
use std::{cell::RefCell, ops::Range, rc::Rc};

pub trait UniformData: bytemuck::NoUninit {}

impl<T> UniformData for T where T: bytemuck::NoUninit {}

#[derive(Debug, Default)]
struct SsboPoolInner {
    ssbo_map: TypeIdMap<ManagedSsbo>,
    ssbo_ids: IdPool,
}

impl SsboPoolInner {
    fn upload_uniform<T: bytemuck::NoUninit>(&mut self, uniform: &ObjectData, val: &T) {
        let entry = self.get_ssbo::<T>();

        let slice = bytemuck::bytes_of(val);

        match entry.find_by_id(uniform.id) {
            Ok(index) => {
                let offset = entry.entries[index].offset_in_buffer;
                entry.staging_buffer[offset..offset + entry.type_info.size].copy_from_slice(slice);
            }
            Err(_) => {
                let offset = entry.staging_buffer.len();
                entry.staging_buffer.extend_from_slice(slice);
                entry.entries.insert(BufferEntry {
                    id: uniform.id,
                    offset_in_buffer: offset,
                });
            }
        }
    }

    fn get_ssbo<T: bytemuck::NoUninit>(&mut self) -> &mut ManagedSsbo {
        let key = TypeId::new::<T>();

        self.ssbo_map.entry(key).or_insert_with(|| ManagedSsbo::new::<T>())
    }

    fn remove_uniform(&mut self, id: InstanceId) {
        // TODO: free buffers
        for data in self.ssbo_map.values_mut() {
            if let Ok(index) = data.find_by_id(id) {
                data.entries.remove_index(index);
                return;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SsboPool {
    inner: Rc<RefCell<SsboPoolInner>>,
}

impl SsboPool {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(SsboPoolInner::default())),
        }
    }

    fn upload_uniform<T: bytemuck::NoUninit>(&self, uniform: &ObjectData, val: &T) {
        self.inner.borrow_mut().upload_uniform(uniform, val);
    }

    fn remove_uniform(&mut self, id: InstanceId) {
        self.inner.borrow_mut().remove_uniform(id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BufferEntry {
    id: InstanceId,
    offset_in_buffer: usize,
}

impl PartialOrd for BufferEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BufferEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug)]
struct ManagedSsbo {
    type_info: TypeInfo,
    updated_range: Range<usize>,
    entries: SortedVec<BufferEntry>,
    staging_buffer: StagingBuffer,
    buffer: wgpu::Buffer,
}

impl ManagedSsbo {
    fn new<T: 'static>() -> Self {
        Self {
            type_info: TypeInfo::new::<T>(),
            updated_range: 0..0,
            entries: SortedVec::new(),
            staging_buffer: vec![],
            buffer: todo!(),
        }
    }

    fn find_by_id(&self, id: InstanceId) -> Result<usize, usize> {
        self.entries.binary_search_by_key(&id, |entry| entry.id)
    }
}

#[derive(Debug)]
pub struct ObjectData {
    id: InstanceId,
    ssbo_pool: SsboPool,
    counter: InstanceCounter,
}

impl ObjectData {
    pub fn upload<T: bytemuck::NoUninit>(&self, val: &T) {
        self.ssbo_pool.upload_uniform(self, val);
    }
}

impl Clone for ObjectData {
    fn clone(&self) -> Self {
        self.counter.update(|v| v + 1);

        Self {
            id: self.id,
            ssbo_pool: self.ssbo_pool.clone(),
            counter: self.counter.clone(),
        }
    }
}

impl Drop for ObjectData {
    fn drop(&mut self) {
        self.counter.update(|v| v - 1);

        if self.counter.get() == 0 {
            self.ssbo_pool.remove_uniform(self.id);
        }
    }
}
