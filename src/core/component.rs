use std::{alloc::Layout, any::TypeId, collections::HashMap, fmt::Debug};

pub trait Component: 'static {}

#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ComponentId(usize);

impl ComponentId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

impl From<usize> for ComponentId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl From<ComponentId> for usize {
    fn from(id: ComponentId) -> Self {
        id.0
    }
}

impl From<&ComponentId> for usize {
    fn from(id: &ComponentId) -> Self {
        id.0
    }
}

impl std::ops::Deref for ComponentId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ComponentId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        TypeId::of::<Self>().fmt(f)
    }
}

pub struct ComponentMeta {
    name: &'static str,
    layout: Layout,
    type_id: TypeId,
}

impl ComponentMeta {
    pub fn new<T: Component>() -> Self {
        Self {
            name: std::any::type_name::<T>(),
            layout: Layout::new::<T>(),
            type_id: TypeId::of::<T>(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

pub struct Components {
    components: Vec<ComponentMeta>,
    id_map: HashMap<TypeId, usize>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            id_map: HashMap::new(),
        }
    }

    pub fn register<T: Component>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<T>();
        let id = self.components.len();
        self.components.push(ComponentMeta::new::<T>());
        self.id_map.insert(type_id, id);
        ComponentId::new(id)
    }

    pub fn get(&self, id: ComponentId) -> &ComponentMeta {
        &self.components[usize::from(id)]
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ComponentMeta> {
        self.components.iter()
    }

    pub fn contains<T: Component>(&self) -> bool {
        self.id_map.contains_key(&TypeId::of::<T>())
    }

    pub fn get_id<T: Component>(&self) -> Option<ComponentId> {
        self.id_map
            .get(&TypeId::of::<T>())
            .map(|id| ComponentId::new(*id))
    }
}
