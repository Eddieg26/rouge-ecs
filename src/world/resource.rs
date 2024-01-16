use crate::storage::ptr::Ptr;
use std::{
    alloc::Layout,
    any::TypeId,
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
    ptr::NonNull,
};

pub trait Resource: Send + Sync + 'static {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceType(u64);

impl ResourceType {
    pub fn new<T: Resource>() -> Self {
        Self(hash_id(&TypeId::of::<T>()))
    }

    pub fn dynamic(value: u64) -> Self {
        Self(value)
    }

    pub fn is<T: Resource>(&self) -> bool {
        self.0 == hash_id(&TypeId::of::<T>())
    }
}

impl From<&TypeId> for ResourceType {
    fn from(type_id: &TypeId) -> Self {
        Self(hash_id(type_id))
    }
}

impl From<TypeId> for ResourceType {
    fn from(type_id: TypeId) -> Self {
        Self(hash_id(&type_id))
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        TypeId::of::<Self>().fmt(f)
    }
}

fn hash_id(id: &std::any::TypeId) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}

pub struct Resources {
    resources: HashMap<ResourceType, ResourceData>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.resources
            .insert(ResourceType::new::<R>(), ResourceData::new(resource));
    }

    pub fn get<R: Resource>(&self) -> &R {
        let ty = ResourceType::new::<R>();
        let res = self.resources.get(&ty).expect("Resource doesn't exist.");
        res.get::<R>()
    }

    pub fn get_mut<R: Resource>(&self) -> &mut R {
        let ty = ResourceType::new::<R>();
        let res = self.resources.get(&ty).expect("Resource doesn't exist.");

        res.get_mut::<R>()
    }
}

pub struct ResourceData {
    data: NonNull<u8>,
    layout: Layout,
}

impl ResourceData {
    pub fn new<R: Resource>(resource: R) -> Self {
        let layout = Layout::new::<R>();
        let data = unsafe { std::alloc::alloc(layout) };

        let data = unsafe {
            std::ptr::write(data as *mut R, resource);
            NonNull::new_unchecked(data)
        };

        ResourceData { data, layout }
    }

    pub fn ptr<'a>(&'a self) -> Ptr<'a> {
        Ptr::new(self.data, self.layout, self.layout.size())
    }

    pub fn get<R: Resource>(&self) -> &R {
        unsafe { &*(self.data.as_ptr() as *const R) }
    }

    pub fn get_mut<R: Resource>(&self) -> &mut R {
        unsafe { &mut *(self.data.as_ptr() as *mut R) }
    }
}

impl Drop for ResourceData {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.data.as_ptr(),
                Layout::from_size_align_unchecked(self.layout.size(), self.layout.align()),
            );
        }
    }
}
