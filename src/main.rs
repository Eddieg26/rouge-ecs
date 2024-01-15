use crate::world::actions::builtin::{DeleteEntity, RemoveComponent};
use core::{Component, Entity};
use schedule::SchedulePhase;
use system::action::ActionSystems;
use world::{
    actions::{
        builtin::{AddComponent, CreateEntity},
        Actions,
    },
    query::Query,
    World,
};

pub mod archetype;
pub mod core;
pub mod schedule;
pub mod storage;
pub mod system;
pub mod world;

pub struct Update;

impl SchedulePhase for Update {
    const PHASE: &'static str = "update";
}

pub struct PostUpdate;
impl SchedulePhase for PostUpdate {
    const PHASE: &'static str = "post_update";
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

fn main() {
    let mut world = World::new();
    world.register::<Player>();
    world.add_system(Update, |actions: &mut Actions| {
        println!("Hello, world!");
        actions.add(CreateEntity::new().with(Player::new(500)));
        actions.add(CreateEntity::new());
    });

    world.add_system(PostUpdate, |actions: &mut Actions| {
        actions.add(DeleteEntity::new(Entity::new(0, 0)));
    });

    let add_player_systems = ActionSystems::<AddComponent<Player>>::new().add_system(
        |entities: &[Entity], q: Query<&Player>| {
            for player in q.entities(entities) {
                println!("Player{:?}", player);
            }
        },
    );

    let remove_player_systems =
        ActionSystems::<RemoveComponent<Player>>::new().add_system(|entities: &[Entity]| {
            for entity in entities {
                println!("Off Player{:?}", entity);
            }
        });

    world.add_action_systems::<AddComponent<Player>>(add_player_systems);
    world.add_action_systems::<RemoveComponent<Player>>(remove_player_systems);

    world.run::<Update>();
    world.run::<PostUpdate>();
    println!("DONE");
}
