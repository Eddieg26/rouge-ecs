use std::{
    alloc::Layout,
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
    ptr::NonNull,
};

use crate::storage::blob::Ptr;

pub trait Resource: 'static + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

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

    pub fn get<R: Resource>(&self) -> Res<R> {
        self.try_get::<R>().expect("Resource doesn't exist.")
    }

    pub fn get_mut<R: Resource>(&self) -> ResMut<R> {
        self.try_get_mut::<R>().expect("Resource doesn't exist.")
    }

    pub fn try_get<R: Resource>(&self) -> Option<Res<R>> {
        Some(self.resources.get(&ResourceType::new::<R>())?.res::<R>())
    }

    pub fn try_get_mut<R: Resource>(&self) -> Option<ResMut<R>> {
        Some(
            self.resources
                .get(&ResourceType::new::<R>())?
                .res_mut::<R>(),
        )
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

    pub fn get<'a>(&'a self) -> Ptr<'a> {
        Ptr::new(self.data, self.layout, self.layout.size())
    }

    pub fn res<'a, R: Resource>(&'a self) -> Res<'a, R> {
        let resource: &R = unsafe { &*(self.data.as_ptr() as *const R) };
        Res::new(resource)
    }

    pub fn res_mut<'a, R: Resource>(&'a self) -> ResMut<'a, R> {
        let resource: &mut R = unsafe { &mut *(self.data.as_ptr() as *mut R) };
        ResMut::new(resource)
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

pub struct Res<'a, T: Resource> {
    resource: &'a T,
}

impl<'a, T: Resource> Res<'a, T> {
    pub fn new(resource: &'a T) -> Self {
        Self { resource }
    }
}

impl<T> std::ops::Deref for Res<'_, T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

pub struct ResMut<'a, T: Resource> {
    resource: &'a mut T,
}

impl<'a, T: Resource> ResMut<'a, T> {
    pub fn new(resource: &'a mut T) -> Self {
        Self { resource }
    }
}

impl<T> std::ops::Deref for ResMut<'_, T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<T> std::ops::DerefMut for ResMut<'_, T>
where
    T: Resource,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource
    }
}
