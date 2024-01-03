use self::resource::{Resource, Resources};

pub mod resource;

pub struct World {
    resources: Resources,
}

impl World {
    pub fn new() -> Self {
        Self {
            resources: Resources::new(),
        }
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(resource);
    }
}
