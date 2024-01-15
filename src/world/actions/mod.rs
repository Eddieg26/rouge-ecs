use super::{resource::Resource, World};
use crate::{
    storage::sparse::SparseMap,
    system::action::{ActionSystem, ActionSystems},
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

pub mod builtin;

pub type ActionSystemVec = Box<dyn Any>;

pub trait Action: 'static + Debug {
    type Output;
    const PRIORITY: u32 = 0;

    fn execute(&mut self, world: &mut World) -> Self::Output;

    fn skip(&self, _: &World) -> bool {
        false
    }

    fn finish(_world: &mut World) {}
}

pub struct ActionSystemExecutor {
    executor: Box<dyn Fn(&mut [Box<dyn Any>], &mut ActionSystemVec, &World)>,
    systems: ActionSystemVec,
    priority: u32,
}

impl ActionSystemExecutor {
    pub fn new<A: Action>() -> Self {
        Self {
            executor: Box::new(move |outputs, systems, world| {
                let mut out = vec![];
                for outputs in outputs {
                    if let Some(outputs) = outputs.downcast_mut::<Vec<A::Output>>() {
                        out.append(outputs);
                    }
                }

                if !out.is_empty() {
                    let systems = systems.downcast_mut::<Vec<ActionSystem<A>>>().unwrap();

                    for system in systems {
                        system.run(&out, world);
                    }
                }
            }),
            systems: Box::new(Vec::<ActionSystem<A>>::new()),
            priority: A::PRIORITY,
        }
    }

    pub fn add_system<A: Action>(&mut self, system: ActionSystem<A>) {
        let systems = self.systems.downcast_mut::<Vec<ActionSystem<A>>>().unwrap();
        systems.push(system);
    }

    pub fn add_systems<A: Action>(&mut self, systems: &mut Vec<ActionSystem<A>>) {
        let owned = self.systems.downcast_mut::<Vec<ActionSystem<A>>>().unwrap();
        owned.append(systems);
    }

    pub fn execute(&mut self, outputs: &mut [Box<dyn Any>], world: &World) {
        (self.executor)(outputs, &mut self.systems, world);
    }
}

pub struct Actions {
    actions: HashMap<TypeId, Box<dyn Any>>,
    appenders: HashMap<TypeId, Box<dyn Fn(&mut Box<dyn Any>, &mut Box<dyn Any>)>>,
    executors: HashMap<TypeId, Box<dyn Fn(&mut World, &mut Box<dyn Any>, &mut ActionOutputs)>>,
    actions_by_priority: Vec<(TypeId, u32)>,
}

impl Actions {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            appenders: HashMap::new(),
            executors: HashMap::new(),
            actions_by_priority: Vec::new(),
        }
    }

    pub fn add<A: Action>(&mut self, action: A) -> &mut Self {
        let type_id = TypeId::of::<A>();
        if let Some(actions) = self.actions.get_mut(&type_id) {
            let actions = actions.downcast_mut::<Vec<A>>().unwrap();
            actions.push(action);
        } else {
            let mut actions = Vec::<A>::with_capacity(1);
            let appender = |actions: &mut Box<dyn Any>, other: &mut Box<dyn Any>| {
                let actions = actions.downcast_mut::<Vec<A>>().unwrap();
                let mut other = other.downcast_mut::<Vec<A>>().unwrap();
                actions.append(&mut other);
            };

            let executor =
                |world: &mut World, actions: &mut Box<dyn Any>, outputs: &mut ActionOutputs| {
                    let actions = actions.downcast_mut::<Vec<A>>().unwrap();

                    for mut action in actions.drain(..) {
                        if !action.skip(world) {
                            outputs.add::<A>(action.execute(world));
                        }
                    }

                    actions.clear();
                    A::finish(world);
                };

            actions.push(action);

            self.actions.insert(type_id, Box::new(actions));
            self.appenders.insert(type_id, Box::new(appender));
            self.executors.insert(type_id, Box::new(executor));
            self.actions_by_priority.push((type_id, A::PRIORITY));
            self.actions_by_priority
                .sort_by(|(_, a), (_, b)| a.cmp(b).reverse());
        }

        self
    }

    pub fn append(&mut self, mut actions: Actions) {
        for (ty, mut actions) in actions.actions.drain() {
            if let Some(appender) = self.appenders.get_mut(&ty) {
                let current = self.actions.get_mut(&ty).unwrap();
                appender(current, &mut actions);
            } else {
                self.actions.insert(ty, actions);
            }
        }
    }

    pub fn take(&mut self) -> Self {
        let mut actions = Self::new();
        std::mem::swap(&mut actions, self);
        actions
    }

    pub fn execute(&mut self, world: &mut World) -> ActionOutputs {
        let mut outputs = ActionOutputs::new();

        for (ty, _) in self.actions_by_priority.iter() {
            if let Some(mut actions) = self.actions.remove(ty) {
                if let Some(executor) = self.executors.get(ty) {
                    executor(world, &mut actions, &mut outputs);
                }
            }
        }

        self.actions.clear();

        outputs
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }
}

pub struct ActionExecutors {
    executors: SparseMap<TypeId, ActionSystemExecutor>,
}

impl ActionExecutors {
    pub fn new() -> Self {
        Self {
            executors: SparseMap::new(),
        }
    }

    pub fn add_system<A: Action>(&mut self, system: ActionSystem<A>) {
        if let Some(executor) = self.executors.get_mut(&TypeId::of::<A>()) {
            executor.add_system(system);
        } else {
            let mut executor = ActionSystemExecutor::new::<A>();
            executor.add_system(system);
            self.executors.insert(TypeId::of::<A>(), executor);
            self.executors
                .sort(|(_, a), (_, b)| a.priority.cmp(&b.priority).reverse());
        }
    }

    pub fn add_systems<A: Action>(&mut self, mut systems: ActionSystems<A>) {
        let mut systems = systems.take();
        if let Some(executor) = self.executors.get_mut(&TypeId::of::<A>()) {
            executor.add_systems(&mut systems);
        } else {
            let mut executor = ActionSystemExecutor::new::<A>();
            executor.add_systems(&mut systems);
            self.executors.insert(TypeId::of::<A>(), executor);
            self.executors
                .sort(|(_, a), (_, b)| a.priority.cmp(&b.priority).reverse());
        }
    }

    pub fn take(&mut self) -> Self {
        let mut executors = Self::new();
        std::mem::swap(&mut executors, self);
        executors
    }

    pub fn execute(&mut self, outputs: &mut [ActionOutputs], world: &World) {
        for (ty, executor) in self.executors.iter_mut() {
            let mut outputs = outputs
                .iter_mut()
                .filter_map(|out| out.outputs.remove(ty))
                .collect::<Vec<_>>();
            if !outputs.is_empty() {
                executor.execute(&mut outputs, world);
            }
        }
    }
}

pub struct ActionOutputs {
    outputs: HashMap<TypeId, Box<dyn Any>>,
}

impl ActionOutputs {
    pub(crate) fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }

    pub(crate) fn take(&mut self) -> Self {
        let mut outputs = Self::new();
        std::mem::swap(&mut outputs, self);
        outputs
    }

    pub fn add<A: Action>(&mut self, output: A::Output) {
        self.outputs
            .insert(TypeId::of::<A>(), Box::new(vec![output]));
    }

    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.outputs.len()
    }
}

impl Default for Actions {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ActionExecutors {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for Actions {}
impl Resource for ActionExecutors {}
impl Resource for ActionOutputs {}
