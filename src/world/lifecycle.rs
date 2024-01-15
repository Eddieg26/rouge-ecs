use crate::{
    archetype::{ArchetypeId, Archetypes},
    core::{Component, ComponentId, Entity},
    storage::{
        blob::Blob,
        sparse::SparseSet,
        table::{Column, Table, TableId, TableRow, Tables},
    },
};
use std::collections::HashMap;

pub struct LifecycleManager {
    created_entities: Vec<Entity>,
    created_components: HashMap<ComponentId, Vec<(Entity, Blob)>>,
    removed_components: HashMap<ComponentId, Vec<Entity>>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            created_entities: Vec::new(),
            created_components: HashMap::new(),
            removed_components: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.created_entities.clear();
        self.created_components.clear();
        self.removed_components.clear();
    }

    pub fn create_entity(&mut self, entity: Entity) {
        self.created_entities.push(entity);
    }

    pub fn add_component<T: Component>(
        &mut self,
        entity: Entity,
        component_id: ComponentId,
        component: T,
    ) {
        let mut blob = Blob::new::<T>();
        blob.push(component);

        self.created_components
            .entry(component_id)
            .or_insert_with(Vec::new)
            .push((entity, blob));
    }

    pub fn remove_component(&mut self, entity: Entity, component_id: ComponentId) {
        self.removed_components
            .entry(component_id)
            .or_insert_with(Vec::new)
            .push(entity);
    }

    pub(crate) fn create_entities(
        &mut self,
        archetypes: &mut Archetypes,
        tables: &mut Tables<Entity>,
    ) {
        let table_id = ArchetypeId::new(&[]).into();
        let table = if let Some(table) = tables.get_mut(table_id) {
            table
        } else {
            let table = Table::<Entity>::with_capacity(self.created_entities.len()).build();
            tables.insert(table);
            tables.get_mut(table_id).unwrap()
        };

        for entity in self.created_entities.drain(..) {
            archetypes.add_entity(entity);
            table.add_row(entity, TableRow::new(entity, SparseSet::new()));
        }
    }

    pub(crate) fn add_components(
        &mut self,
        id: ComponentId,
        archetypes: &mut Archetypes,
        tables: &mut Tables<Entity>,
    ) {
        if let Some(created) = self.created_components.remove(&id) {
            for (entity, blob) in created {
                let archetype = archetypes.archetype_id(entity).cloned().unwrap();
                let new_archetype_id = archetypes.add_component(entity, id).unwrap();

                let old_table_id: TableId = archetype.into();

                let mut row = tables
                    .get_mut(old_table_id)
                    .unwrap()
                    .remove_row(entity)
                    .unwrap();

                row.insert(id.into(), Column::from_blob(blob));

                let new_table_id: TableId = new_archetype_id.into();
                let new_table = if let Some(table) = tables.get_mut(new_table_id) {
                    table
                } else {
                    let table = Table::<Entity>::from_row(&row, 1);
                    tables.insert(table);
                    tables.get_mut(new_table_id).unwrap()
                };

                new_table.add_row(entity, row);
            }
        }
    }

    pub(crate) fn remove_components(
        &mut self,
        id: ComponentId,
        archetypes: &mut Archetypes,
        tables: &mut Tables<Entity>,
    ) {
        if let Some(removed) = self.removed_components.remove(&id) {
            for entity in removed {
                if !archetypes.has(entity, id) {
                    continue;
                }

                let archetype = archetypes.archetype_id(entity).cloned().unwrap();
                let new_archetype_id = archetypes.remove_component(entity, id).unwrap();

                let old_table_id: TableId = archetype.into();

                let mut row = tables
                    .get_mut(old_table_id)
                    .unwrap()
                    .remove_row(entity)
                    .unwrap();

                row.remove(id.into());

                let new_table_id: TableId = new_archetype_id.into();
                let new_table = if let Some(table) = tables.get_mut(new_table_id) {
                    table
                } else {
                    let table = Table::<Entity>::from_row(&row, 1);
                    tables.insert(table);
                    tables.get_mut(new_table_id).unwrap()
                };

                new_table.add_row(entity, row);
            }
        }
    }
}
