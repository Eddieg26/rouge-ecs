pub struct SparseArray<V> {
    values: Vec<Option<V>>,
}

impl<V> SparseArray<V> {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, index: usize, value: V) {
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    pub fn get(&self, index: usize) -> Option<&V> {
        self.values.get(index).map(|value| value.as_ref().unwrap())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut V> {
        self.values
            .get_mut(index)
            .map(|value| value.as_mut().unwrap())
    }

    pub fn remove(&mut self, index: usize) -> Option<V> {
        self.values
            .get_mut(index)
            .map(|value| value.take().unwrap())
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.values.iter().filter_map(|value| value.as_ref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.values.iter_mut().filter_map(|value| value.as_mut())
    }

    pub fn contains(&self, index: usize) -> bool {
        self.values
            .get(index)
            .map(|value| value.is_some())
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn into_immutable(self) -> ImmutableSparseArray<V> {
        ImmutableSparseArray {
            values: self.values.into_boxed_slice(),
        }
    }
}

pub struct SparseSet<V> {
    values: Vec<V>,
    indices: Vec<usize>,
    array: SparseArray<usize>,
}

impl<V> SparseSet<V> {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            indices: Vec::new(),
            array: SparseArray::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
            array: SparseArray::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, index: usize, value: V) -> Option<V> {
        if let Some(mapped_index) = self.array.get(index) {
            self.values[*mapped_index] = value;
        } else {
            let mapped_index = self.values.len();
            self.values.push(value);
            self.indices.push(index);
            self.array.insert(index, mapped_index);
        }

        None
    }

    pub fn get(&self, index: usize) -> Option<&V> {
        self.array
            .get(index)
            .map(|mapped_index| &self.values[*mapped_index])
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut V> {
        self.array
            .get(index)
            .map(|mapped_index| &mut self.values[*mapped_index])
    }

    pub fn remove(&mut self, index: usize) -> Option<V> {
        if let Some(mapped_index) = self.array.remove(index) {
            let value = self.values.swap_remove(mapped_index);
            let index = self.indices.swap_remove(mapped_index);
            self.array.insert(index, mapped_index);
            Some(value)
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.values.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.values.iter_mut()
    }

    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.indices.iter().cloned()
    }

    pub fn contains(&self, index: usize) -> bool {
        self.array.contains(index)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.indices.clear();
        self.array = SparseArray::new();
    }

    pub fn into_immutable(self) -> ImmutableSparseSet<V> {
        ImmutableSparseSet {
            values: self.values.into_boxed_slice(),
            indices: self.indices.into_boxed_slice(),
            array: self.array.into_immutable(),
        }
    }
}

pub struct ImmutableSparseArray<V> {
    values: Box<[Option<V>]>,
}

impl<V> ImmutableSparseArray<V> {
    pub fn get(&self, index: usize) -> Option<&V> {
        self.values.get(index).map(|v| v.as_ref()).unwrap_or(None)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Option<V>> {
        self.values.iter()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub struct ImmutableSparseSet<V> {
    values: Box<[V]>,
    indices: Box<[usize]>,
    array: ImmutableSparseArray<usize>,
}

impl<V> ImmutableSparseSet<V> {
    pub fn get(&self, index: usize) -> Option<&V> {
        self.array
            .get(index)
            .map(|mapped_index| &self.values[*mapped_index])
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.values.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.values.iter_mut()
    }

    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.indices.iter().cloned()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
