use crate::{
    storage::blob::Blob,
    system::observer::{
        builtin::{AddComponent, CreateEntity, DeleteEntity, RemoveComponent},
        Actions, Observers,
    },
};
use core::{Component, Entity};
use schedule::{ScheduleLabel, SchedulePhase};
use storage::bits::BitSet;
use world::{query::Query, World};

pub mod archetype;
pub mod core;
pub mod schedule;
pub mod storage;
pub mod system;
pub mod tasks;
pub mod world;

pub struct Update;

impl SchedulePhase for Update {
    const PHASE: &'static str = "update";
}

pub struct PostUpdate;
impl SchedulePhase for PostUpdate {
    const PHASE: &'static str = "post_update";
}

pub struct DefaultLabel;

impl ScheduleLabel for DefaultLabel {
    const LABEL: &'static str = "default";
}

#[derive(Debug)]
pub struct Player {
    health: u32,
}

impl Player {
    pub fn new(health: u32) -> Self {
        Self { health }
    }

    pub fn health(&self) -> u32 {
        self.health
    }
}

impl Component for Player {}

// fn main() {
//     let mut world = World::new();
//     world.register::<Player>();
//     world.add_system(Update, DefaultLabel, |actions: &mut Actions| {
//         println!("Hello, world!");
//         actions.add(CreateEntity::new().with(Player::new(500)));
//         actions.add(CreateEntity::new());
//     });

//     world.add_system(PostUpdate, DefaultLabel, |actions: &mut Actions| {
//         actions.add(DeleteEntity::new(Entity::new(0, 0)));
//     });

//     let add_player_systems = Observers::<AddComponent<Player>>::new().add_system(
//         |entities: &[Entity], q: Query<&Player>| {
//             for player in q.entities(entities) {
//                 println!("Player{:?}", player);
//             }
//         },
//     );

//     let remove_player_systems =
//         Observers::<RemoveComponent<Player>>::new().add_system(|entities: &[Entity]| {
//             for entity in entities {
//                 println!("Off Player{:?}", entity);
//             }
//         });

//     world.add_observers::<AddComponent<Player>>(add_player_systems);
//     world.add_observers::<RemoveComponent<Player>>(remove_player_systems);

//     world.run::<Update>();
//     world.run::<PostUpdate>();
//     println!("DONE");
// }

pub struct DebugEntity {
    id: u32,
    add_components: Vec<Box<dyn FnMut(Entity, &mut World)>>,
}

impl DebugEntity {
    pub fn new(id: u32) -> Self {
        Self {
            id,
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

impl std::fmt::Debug for DebugEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugEntity").field("id", &self.id).finish()
    }
}

fn main() {
    // let mut actions = Blob::new::<DebugEntity>();
    // println!("BASE: {:?}", actions.layout());
    // println!("ALIGNED: {:?}", actions.aligned_layout());
    // actions.push(DebugEntity::new(0));
    // actions.push(DebugEntity::new(1));
    // actions.push(DebugEntity::new(2));
    // actions.push(DebugEntity::new(3));

    // for action in actions.iter::<DebugEntity>() {
    //     println!("{:?}", action);
    // }

    let mut set = BitSet::with_capacity(10);
    set.set(0);

    set.set(6);
    set.unset(6);
    set.set(6);

    set.set(9);

    println!("SET:");
    for (i, value) in set.iter().enumerate() {
        println!("{}: {}", i, value);
    }
}
