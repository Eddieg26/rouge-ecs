use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
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
    resources: HashMap<ResourceType, ResourceObj>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.resources
            .insert(ResourceType::new::<T>(), ResourceObj::new(resource));
    }

    fn get_obj<T: Resource>(&self) -> &ResourceObj {
        self.resources
            .get(&ResourceType::new::<T>())
            .expect("Resource not found")
    }

    fn get_obj_mut<T: Resource>(&mut self) -> &mut ResourceObj {
        self.resources
            .get_mut(&ResourceType::new::<T>())
            .expect("Resource not found")
    }

    pub fn get<T: Resource>(&self) -> Res<T> {
        Res::new(self.get_obj::<T>().get())
    }

    pub fn get_mut<T: Resource>(&mut self) -> ResMut<T> {
        ResMut::new(self.get_obj_mut::<T>().get_mut())
    }
}

pub struct ResourceObj(Ptr);

impl ResourceObj {
    pub fn new<T: Resource>(resource: T) -> Self {
        Self(Ptr::from_data::<T>(resource))
    }

    pub fn get<T: Resource>(&self) -> &T {
        self.0.get(0)
    }

    pub fn get_mut<T: Resource>(&self) -> &mut T {
        self.0.get_mut(0)
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
