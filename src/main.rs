use crate::system::{
    observer::{
        builtin::{AddComponent, CreateEntity, DeleteEntity, RemoveComponent},
        Actions, Observers,
    },
    IntoSystem,
};
use core::{Component, Entity};
use schedule::{ScheduleLabel, SchedulePhase};

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

fn start(actions: &mut Actions) {
    println!("Start");
    actions.add(CreateEntity::new().with(Player::new(100)));
    actions.add(CreateEntity::new());
}

fn update(actions: &mut Actions) {
    println!("Update");
}

fn post_update(actions: &mut Actions) {
    println!("Post Update");
    actions.add(DeleteEntity::new(Entity::new(0, 0)));
}

fn player_added(entities: &[Entity], q: Query<&Player>) {
    println!("Player Added");
    for player in q.entities(entities) {
        println!("Player{:?}", player);
    }
}

fn player_removed(entities: &[Entity]) {
    println!("Player Removed");
    for entity in entities {
        println!("Off Player{:?}", entity);
    }
}

fn entities_deleted(entities: &[Entity]) {
    println!("Entities Deleted");
    for entity in entities {
        println!("Deleted Entity{:?}", entity);
    }
}

fn main() {
    let mut world = World::new();
    world.register::<Player>();
    world.add_system(Update, DefaultLabel, update.after(start));
    world.add_system(PostUpdate, DefaultLabel, post_update);

    let add_player_systems = Observers::<AddComponent<Player>>::new().add_system(player_added);
    let remove_player_systems =
        Observers::<RemoveComponent<Player>>::new().add_system(player_removed);
    let delete_entity_systems = Observers::<DeleteEntity>::new().add_system(entities_deleted);
    world.add_observers(add_player_systems);
    world.add_observers(remove_player_systems);
    world.add_observers(delete_entity_systems);

    world.build_schedules();
    world.run::<Update>();
    world.run::<PostUpdate>();

    println!("DONE");
}

// #[derive(Debug)]
// pub struct DebugEntity {
//     id: u32,
// }

// impl DebugEntity {
//     pub fn new(id: u32) -> Self {
//         Self { id }
//     }
// }

// #[derive(Debug)]
// pub struct DebugResource {
//     id: u32,
// }

// impl DebugResource {
//     pub fn new(id: u32) -> Self {
//         Self { id }
//     }
// }

// impl Resource for DebugResource {}

// fn main() {
//     let mut actions = Blob::new::<DebugEntity>();
//     println!("BASE: {:?}", actions.layout());
//     println!("ALIGNED: {:?}", actions.aligned_layout());
//     actions.push(DebugEntity::new(0));
//     actions.push(DebugEntity::new(1));
//     actions.push(DebugEntity::new(2));
//     actions.push(DebugEntity::new(3));

//     for action in actions.iter::<DebugEntity>() {
//         println!("{:?}", action);
//     }

//     let mut resources = Resources::new();
//     resources.insert(DebugResource::new(0));

//     let debug = resources.get::<DebugResource>();

//     println!("{:?}", debug);
// }
