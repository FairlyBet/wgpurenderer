use nohash_hasher::{IntMap, IsEnabled};
use std::{any, cell::Cell, rc::Rc};

pub type TypeIdMap<V> = IntMap<TypeId, V>;

pub type StagingBuffer = Vec<u8>;

#[derive(Debug, Clone)]
pub struct InstanceCounter(Rc<Cell<u32>>);

impl InstanceCounter {
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(1)))
    }

    pub fn increment(&self) {
        self.0.update(|v| v + 1);
    }

    pub fn decrement(&self) {
        self.0.update(|v| v - 1);
    }

    pub fn value(&self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(u32);

impl InstanceId {
    pub fn new(val: u32) -> Self {
        Self(val)
    }
}

impl From<u32> for InstanceId {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TypeId(any::TypeId);

impl TypeId {
    pub fn new<T: 'static>() -> Self {
        Self(any::TypeId::of::<T>())
    }
}

impl IsEnabled for TypeId {}

impl From<any::TypeId> for TypeId {
    fn from(id: any::TypeId) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeInfo {
    pub size: usize,
    pub align: usize,
    pub type_id: TypeId,
    pub name: &'static str,
}

impl TypeInfo {
    pub fn new<T: 'static>() -> Self {
        Self {
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            type_id: TypeId::new::<T>(),
            name: std::any::type_name::<T>(),
        }
    }
}

#[derive(Debug, Default)]
pub struct IdPool {
    current: u32,
    available: Vec<InstanceId>,
}

impl IdPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_base(base: u32) -> Self {
        let mut this = Self::new();
        this.current = base;
        this
    }

    pub fn get_next(&mut self) -> InstanceId {
        if let Some(id) = self.available.pop() {
            id
        } else {
            let ret = InstanceId(self.current);
            self.current += 1;
            ret
        }
    }

    pub fn free(&mut self, id: InstanceId) {
        debug_assert!(
            id.0 < self.current,
            "Id {id:?} can't be freed, as it was never created by the pool"
        );
        self.available.push(id);
    }
}
