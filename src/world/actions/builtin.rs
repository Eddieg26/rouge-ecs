use std::fmt::Debug;

use super::{Action, ActionOutputs, Actions};
use crate::{
    core::{Component, Entity},
    world::World,
};

pub struct CreateEntity {
    add_components: Vec<Box<dyn FnMut(Entity, &mut World)>>,
}

impl CreateEntity {
    pub fn new() -> Self {
        Self {
            add_components: Vec::new(),
        }
    }

    pub fn with<C: Component>(mut self, component: C) -> Self {
        let mut component = Box::new(Some(component));
        let add_component = move |entity: Entity, world: &mut World| {
            if let Some(component) = component.take() {
                world
                    .resource_mut::<Actions>()
                    .add(AddComponent::new(entity, component));
            }
        };

        self.add_components.push(Box::new(add_component));

        self
    }
}

impl Debug for CreateEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateEntity").finish()
    }
}

impl Action for CreateEntity {
    type Output = Entity;
    const PRIORITY: u32 = u32::MAX;

    fn execute(&mut self, world: &mut crate::world::World) -> Self::Output {
        let entity = world.spawn();

        for add_component in self.add_components.iter_mut() {
            add_component(entity, world);
        }

        entity
    }
}

pub struct AddComponent<C: Component> {
    entity: Entity,
    component: Option<C>,
}

impl<C: Component> AddComponent<C> {
    pub fn new(entity: Entity, component: C) -> Self {
        Self {
            entity,
            component: Some(component),
        }
    }
}

impl<C: Component> Action for AddComponent<C> {
    type Output = Entity;
    const PRIORITY: u32 = CreateEntity::PRIORITY - 1;

    fn execute(&mut self, world: &mut crate::world::World) -> Self::Output {
        if let Some(component) = self.component.take() {
            world.add_component(self.entity, component);
        }

        self.entity
    }
}

impl<C: Component> Debug for AddComponent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddComponent")
            .field("entity", &self.entity)
            .finish()
    }
}

pub struct RemoveComponent<C: Component> {
    entity: Entity,
    _marker: std::marker::PhantomData<C>,
}

impl<C: Component> RemoveComponent<C> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Component> Debug for RemoveComponent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoveComponent")
            .field("entity", &self.entity)
            .finish()
    }
}

impl<C: Component> Action for RemoveComponent<C> {
    type Output = Entity;
    const PRIORITY: u32 = AddComponent::<C>::PRIORITY - 1;

    fn execute(&mut self, world: &mut crate::world::World) -> Self::Output {
        world.remove_component::<C>(self.entity);

        self.entity
    }

    fn skip(&self, world: &World) -> bool {
        !world.has::<C>(self.entity)
    }
}

#[derive(Debug)]
pub struct DeleteEntity {
    entity: Entity,
}

impl DeleteEntity {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl Action for DeleteEntity {
    type Output = Entity;
    const PRIORITY: u32 = CreateEntity::PRIORITY - 3;

    fn execute(&mut self, world: &mut crate::world::World) -> Self::Output {
        world.delete(self.entity);

        self.entity
    }
}

pub struct ComponentActionMeta {
    on_remove: Box<dyn Fn(&Entity, &mut ActionOutputs)>,
}

impl ComponentActionMeta {
    pub fn new<C: Component>() -> Self {
        Self {
            on_remove: Box::new(|entity, outputs: &mut ActionOutputs| {
                outputs.add::<RemoveComponent<C>>(*entity);
            }),
        }
    }

    pub fn on_remove(&self) -> &dyn Fn(&Entity, &mut ActionOutputs) {
        &self.on_remove
    }
}
