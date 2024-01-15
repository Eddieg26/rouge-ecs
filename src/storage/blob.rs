use super::ptr::Ptr;
use std::{alloc::Layout, marker::PhantomData, ptr::NonNull};

pub struct Blob {
    capacity: usize,
    len: usize,
    layout: Layout,
    aligned_layout: Layout,
    data: NonNull<u8>,
    drop: Option<fn(*mut u8)>,
}

impl Blob {
    pub fn new<T>() -> Self {
        let base_layout = Layout::new::<T>();
        let aligned_layout = Self::align_layout(&base_layout);
        let data = unsafe { std::alloc::alloc(aligned_layout) };

        let drop = if std::mem::needs_drop::<T>() {
            Some(drop::<T> as fn(*mut u8))
        } else {
            None
        };

        Self {
            capacity: 1,
            len: 0,
            layout: base_layout,
            aligned_layout,
            data: NonNull::new(data).unwrap(),
            drop,
        }
    }

    pub fn with_capacity<T>(capacity: usize) -> Self {
        let base_layout = Layout::new::<T>();
        let aligned_layout = Self::align_layout(&base_layout);
        let data = unsafe { std::alloc::alloc(aligned_layout) };

        let drop = if std::mem::needs_drop::<T>() {
            Some(drop::<T> as fn(*mut u8))
        } else {
            None
        };

        Self {
            capacity,
            len: 0,
            layout: base_layout,
            aligned_layout,
            data: NonNull::new(data).unwrap(),
            drop,
        }
    }

    pub fn copy(&self, capacity: usize) -> Self {
        Blob {
            capacity,
            len: 0,
            layout: self.layout,
            aligned_layout: self.aligned_layout,
            data: NonNull::new(unsafe {
                std::alloc::alloc(Layout::from_size_align_unchecked(
                    self.aligned_layout.size() * capacity,
                    self.aligned_layout.align(),
                ))
            })
            .unwrap(),
            drop: self.drop.clone(),
        }
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn aligned_layout(&self) -> &Layout {
        &self.aligned_layout
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn drop_fn(&self) -> &Option<fn(*mut u8)> {
        &self.drop
    }

    pub fn iter<T: 'static>(&self) -> BlobIterator<T> {
        BlobIterator {
            blob: self,
            current: 0,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut<T: 'static>(&self) -> BlobMutIterator<T> {
        BlobMutIterator {
            blob: self,
            current: 0,
            _marker: PhantomData,
        }
    }

    pub fn to_vec<T: 'static>(&mut self) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len);
        while let Some(value) = self.pop() {
            vec.push(value);
        }

        vec
    }

    pub fn clear(&mut self) {
        self.drop_all();
        self.dealloc();
    }

    pub fn push<T>(&mut self, value: T) {
        if self.len >= self.capacity {
            self.grow();
        }

        unsafe {
            let dst = self.offset(self.len) as *mut T;

            std::ptr::write(dst, value);
        }

        self.len += 1;
    }

    pub fn pop<T>(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            unsafe {
                let ptr = self.offset(self.len) as *mut T;
                let data = std::ptr::read(ptr);

                Some(data)
            }
        } else {
            None
        }
    }

    pub fn extend<T>(&mut self, values: &[T]) {
        if self.len + values.len() > self.capacity {
            self.grow_exact(self.len + values.len());
        }

        unsafe {
            let dst = self.offset(self.len) as *mut T;
            std::ptr::copy_nonoverlapping(values.as_ptr(), dst, values.len());
        }

        self.len += values.len();
    }

    pub fn append(&mut self, other: &mut Blob) {
        if self.len + other.len > self.capacity {
            self.grow_exact(self.len + other.len);
        }

        unsafe {
            let dst = self.offset(self.len) as *mut u8;
            let src = other.data.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, other.aligned_layout.size() * other.len);
        }

        self.len += other.len;
        other.dealloc();
    }

    pub fn swap_remove(&mut self, index: usize) -> Blob {
        unsafe {
            let mut blob = self.copy(1);

            let src = self.offset(index);
            let dst = blob.data.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, self.aligned_layout.size());

            self.len -= 1;

            blob
        }
    }

    pub fn replace<T>(&mut self, index: usize, value: T) -> Option<T> {
        if index < self.len {
            unsafe {
                let src = self.offset(index) as *mut T;
                let mut old = std::ptr::read(src);
                std::mem::replace(&mut old, value);
                Some(old)
            }
        } else {
            None
        }
    }

    pub fn ptr<'a>(&'a self) -> Ptr<'a> {
        Ptr::new(self.data, self.aligned_layout, self.len)
    }

    pub fn get<T>(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(unsafe { &*(self.offset(index) as *const T) })
        } else {
            None
        }
    }

    pub fn get_mut<T>(&self, index: usize) -> Option<&mut T> {
        if index < self.len {
            Some(unsafe { &mut *(self.offset(index) as *mut T) })
        } else {
            None
        }
    }
}

impl Blob {
    fn align_layout(layout: &Layout) -> Layout {
        let align = if layout.align().is_power_of_two() {
            layout.align()
        } else {
            layout.align().next_power_of_two()
        };

        let size = layout.size();
        let padding = (align - (size % align)) % align;

        unsafe { Layout::from_size_align_unchecked(size + padding, align) }
    }

    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        self.grow_exact(new_capacity);
    }

    fn grow_exact(&mut self, new_capacity: usize) {
        if self.capacity >= new_capacity {
            return;
        }

        let new_data = unsafe {
            std::alloc::realloc(
                self.data.as_ptr(),
                self.aligned_layout,
                new_capacity * self.aligned_layout.size(),
            )
        };

        self.capacity = new_capacity;
        self.data = NonNull::new(new_data).unwrap();
    }

    fn offset(&self, index: usize) -> *mut u8 {
        unsafe { self.data.as_ptr().add(index * self.aligned_layout.size()) }
    }

    fn dealloc(&mut self) {
        unsafe {
            let value = std::alloc::realloc(
                self.data.as_ptr(),
                self.aligned_layout,
                self.aligned_layout.size(),
            );

            let layout = Layout::from_size_align_unchecked(
                self.capacity * self.aligned_layout.size(),
                self.aligned_layout.align(),
            );
            std::alloc::dealloc(value, layout);
        };

        self.len = 0;
        self.capacity = 0;
    }

    fn drop_all(&mut self) {
        for i in 0..self.len {
            let ptr = unsafe { self.data.as_ptr().add(i * self.aligned_layout.size()) };
            if let Some(drop) = &self.drop {
                drop(ptr as *mut u8);
            }
        }

        self.len = 0;
    }
}

fn drop<T>(data: *mut u8) {
    unsafe {
        let raw = data as *mut T;
        std::mem::drop(raw.read());
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        unsafe {
            if self.capacity > 0 {
                for i in 0..self.len {
                    let ptr = self.data.as_ptr().add(i * self.aligned_layout.size());
                    if let Some(drop) = &self.drop {
                        drop(ptr);
                    }
                }

                let layout = Layout::from_size_align_unchecked(
                    self.capacity * self.aligned_layout.size(),
                    self.aligned_layout.align(),
                );

                std::alloc::dealloc(self.data.as_ptr(), layout);
            }
        }
    }
}

pub struct BlobIterator<'a, T> {
    blob: &'a Blob,
    current: usize,
    _marker: PhantomData<T>,
}

impl<'a, T: 'static> Iterator for BlobIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.blob.len {
            let value = self.blob.get::<T>(self.current);
            self.current += 1;
            value
        } else {
            None
        }
    }
}

pub struct BlobMutIterator<'a, T> {
    blob: &'a Blob,
    current: usize,
    _marker: PhantomData<T>,
}

impl<'a, T: 'static> Iterator for BlobMutIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.blob.len {
            let value = self.blob.get_mut::<T>(self.current);
            self.current += 1;
            value
        } else {
            None
        }
    }
}
