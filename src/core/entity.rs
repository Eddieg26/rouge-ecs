use super::{GenId, IdAllocator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    id: usize,
    generation: u32,
}

impl Entity {
    pub fn new(id: usize, generation: u32) -> Self {
        Self { id, generation }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

impl Into<GenId> for Entity {
    fn into(self) -> GenId {
        GenId::new(self.id, self.generation)
    }
}

pub struct Entities {
    allocator: IdAllocator,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            allocator: IdAllocator::new(),
        }
    }

    pub fn create(&mut self) -> Entity {
        let id = self.allocator.allocate();
        Entity::new(id.id(), id.generation())
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.allocator
            .free(GenId::new(entity.id(), entity.generation()));
    }

    pub fn reserve(&mut self, amount: usize) {
        self.allocator.reserve(amount);
    }

    pub fn len(&self) -> usize {
        self.allocator.len()
    }

    pub fn is_empty(&self) -> bool {
        self.allocator.is_empty()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.allocator
            .is_alive(GenId::new(entity.id(), entity.generation()))
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.allocator
            .iter()
            .map(|id| Entity::new(id.id(), id.generation()))
    }
}
