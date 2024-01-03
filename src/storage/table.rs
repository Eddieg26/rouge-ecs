use crate::core::Entity;

use super::{
    blob::{Blob, Ptr},
    sparse::ImmutableSparseSet,
};

pub struct Column {
    data: Blob,
}

impl Column {
    pub fn new<T>() -> Self {
        Self {
            data: Blob::new::<T>(),
        }
    }

    pub fn with_capacity<T>(capacity: usize) -> Self {
        Self {
            data: Blob::with_capacity::<T>(capacity),
        }
    }

    pub fn push<T>(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn get(&self, index: usize) -> Option<Ptr> {
        if index < self.data.len() {
            Some(self.data.ptr().add(index))
        } else {
            None
        }
    }

    pub fn ptr(&self) -> Ptr {
        self.data.ptr()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

pub struct Table {
    columns: ImmutableSparseSet<Column>,
    entity: Vec<Entity>,
}
