use super::{
    blob::{Blob, Ptr},
    sparse::{ImmutableSparseSet, SparseSet},
};
use crate::core::IntoGenId;
use std::alloc::Layout;

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

    pub fn from_layout(layout: Layout, capacity: usize) -> Self {
        Self {
            data: Blob::from_layout(layout, capacity),
        }
    }

    pub fn push<T>(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn swap_remove(&mut self, index: usize) {
        self.data.swap_remove(index);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Row(usize);

impl Row {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

impl std::ops::Deref for Row {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Row {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct TableBuilder<I: IntoGenId> {
    columns: SparseSet<Column>,
    capacity: usize,
    _marker: std::marker::PhantomData<I>,
}

impl<I: IntoGenId> TableBuilder<I> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            columns: SparseSet::with_capacity(capacity),
            capacity,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_column(mut self, index: usize, layout: Layout) -> Self {
        self.columns
            .insert(index, Column::from_layout(layout, self.capacity));

        self
    }

    pub fn build(self) -> Table<I> {
        Table {
            columns: self.columns.into_immutable(),
            rows: Vec::with_capacity(self.capacity),
            sparse: SparseSet::with_capacity(self.capacity),
        }
    }
}

pub struct Table<I: IntoGenId> {
    columns: ImmutableSparseSet<Column>,
    rows: Vec<I>,
    sparse: SparseSet<Row>,
}

impl<I: IntoGenId> Table<I> {
    pub fn with_capacity(capacity: usize) -> TableBuilder<I> {
        TableBuilder::with_capacity(capacity)
    }

    pub fn add_row<V>(&mut self, id: I) -> Row {
        let index = self.rows.len();
        let row = Row::new(index);

        self.rows.push(id);
        self.sparse.insert(index, row);

        row
    }

    pub fn remove_row(&mut self, row: Row) -> Option<I> {
        let index = *row;

        if index < self.rows.len() {
            let id = self.rows.swap_remove(index);
            self.sparse.remove(index);
            self.columns
                .iter_mut()
                .for_each(|column| column.swap_remove(index));

            Some(id)
        } else {
            None
        }
    }
}
