use std::{alloc::Layout, marker::PhantomData, ptr::NonNull};

pub struct Ptr<'a> {
    data: NonNull<u8>,
    layout: Layout,
    size: usize,
    _marker: &'a PhantomData<()>,
}

impl<'a> Ptr<'a> {
    pub fn new(data: NonNull<u8>, layout: Layout, size: usize) -> Self {
        Self {
            data,
            layout,
            size,
            _marker: &PhantomData,
        }
    }

    pub fn from_data<T>(data: T) -> Self {
        let data = NonNull::new(&data as *const T as *mut u8).unwrap();
        Self {
            data,
            layout: Layout::new::<T>(),
            size: 1,
            _marker: &PhantomData,
        }
    }

    pub fn offset(&self, offset: usize) -> Self {
        Self {
            data: unsafe { NonNull::new_unchecked(self.data.as_ptr().add(offset)) },
            layout: self.layout,
            size: self.size - offset,
            _marker: &PhantomData,
        }
    }

    pub fn add(&self, index: usize) -> Self {
        Self {
            data: unsafe {
                NonNull::new_unchecked(self.data.as_ptr().add(index * self.layout.size()))
            },
            layout: self.layout,
            size: self.size - (index * self.layout.size()),
            _marker: &PhantomData,
        }
    }

    pub fn get<T>(&self, index: usize) -> &T {
        unsafe { &*(self.data.as_ptr().add(index * self.layout.size()) as *const T) }
    }

    pub fn get_mut<T>(&self, index: usize) -> &mut T {
        unsafe { &mut *(self.data.as_ptr().add(index * self.layout.size()) as *mut T) }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.data.as_ptr()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

pub struct Blob {
    capacity: usize,
    len: usize,
    layout: Layout,
    aligned_layout: Layout,
    data: NonNull<u8>,
}

impl Blob {
    pub fn new<T>() -> Self {
        let base_layout = Layout::new::<T>();
        let aligned_layout = Self::align_layout(&base_layout);
        let data = unsafe { std::alloc::alloc(aligned_layout) };

        Self {
            capacity: 1,
            len: 0,
            layout: base_layout,
            aligned_layout,
            data: NonNull::new(data).unwrap(),
        }
    }

    pub fn from_layout(layout: Layout, capacity: usize) -> Self {
        let aligned_layout = Self::align_layout(&layout);
        let data = unsafe {
            std::alloc::alloc(Layout::from_size_align_unchecked(
                aligned_layout.size() * capacity,
                aligned_layout.align(),
            ))
        };

        Self {
            capacity,
            len: 0,
            layout,
            aligned_layout,
            data: NonNull::new(data).unwrap(),
        }
    }

    pub fn with_capacity<T>(capacity: usize) -> Self {
        let base_layout = Layout::new::<T>();
        let aligned_layout = Self::align_layout(&base_layout);
        let data = unsafe { std::alloc::alloc(aligned_layout) };

        Self {
            capacity,
            len: 0,
            layout: base_layout,
            aligned_layout,
            data: NonNull::new(data).unwrap(),
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

    pub fn clear(&mut self) {
        self.len = 0;
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

        self.capacity = self.aligned_layout.size();
    }

    pub fn push<T>(&mut self, value: T) {
        if self.len >= self.capacity {
            self.grow();
        }

        unsafe {
            let ptr = self
                .data
                .as_ptr()
                .add(self.len * self.aligned_layout.size()) as *mut T;
            std::ptr::write(ptr, value);
        }

        self.len += 1;
    }

    pub fn swap_remove(&mut self, index: usize) {
        unsafe {
            let ptr = self.data.as_ptr().add(index * self.aligned_layout.size());
            std::ptr::copy(ptr, ptr, (self.len - index - 1) * self.layout.size());
        }

        self.len -= 1;
    }

    pub fn replace<T>(&mut self, index: usize, value: T) {
        unsafe {
            let ptr = self.data.as_ptr().add(index * self.aligned_layout.size()) as *mut T;
            std::ptr::write(ptr, value);
        }
    }

    pub fn ptr<'a>(&'a self) -> Ptr<'a> {
        Ptr::new(self.data, self.aligned_layout, self.len)
    }

    pub fn get<T>(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(unsafe {
                &*(self.data.as_ptr().add(index * self.aligned_layout.size()) as *const T)
            })
        } else {
            None
        }
    }

    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
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
}

impl Drop for Blob {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.data.as_ptr(),
                Layout::from_size_align_unchecked(self.capacity, self.aligned_layout.align()),
            );
        }
    }
}
