use super::{ArgItem, SystemArg};
use crate::{
    storage::{blob::Blob, sparse::SparseMap},
    world::{meta::AccessMeta, resource::Resource, World},
};
use std::any::TypeId;

pub mod action;
pub mod builtin;

pub use action::*;

pub struct Observer<A: Action> {
    function: Box<dyn Fn(&[A::Output], &World)>,
    reads: Vec<TypeId>,
    writes: Vec<TypeId>,
}

impl<A: Action> Observer<A> {
    fn new(
        function: impl Fn(&[A::Output], &World) + 'static,
        reads: Vec<TypeId>,
        writes: Vec<TypeId>,
    ) -> Self {
        Self {
            function: Box::new(function),
            reads,
            writes,
        }
    }

    pub fn reads(&self) -> &[TypeId] {
        &self.reads
    }

    pub fn writes(&self) -> &[TypeId] {
        &self.writes
    }

    pub fn run(&mut self, outputs: &[A::Output], world: &World) {
        (self.function)(outputs, world);
    }
}

pub struct Observers<A: Action> {
    systems: Vec<Observer<A>>,
}

impl<A: Action> Observers<A> {
    pub fn new() -> Self {
        Self { systems: vec![] }
    }

    pub fn add_system<M>(mut self, system: impl IntoObserver<A, M>) -> Self {
        self.systems.push(system.into_observer());

        self
    }

    pub fn take(&mut self) -> Vec<Observer<A>> {
        std::mem::take(&mut self.systems)
    }
}

pub trait IntoObserver<A: Action, M> {
    fn into_observer(self) -> Observer<A>;
}

impl<A: Action, F> IntoObserver<A, F> for F
where
    F: Fn(&[A::Output]) + 'static,
{
    fn into_observer(self) -> Observer<A> {
        Observer::new(
            move |outputs: &[A::Output], _: &World| {
                (self)(outputs);
            },
            vec![],
            vec![],
        )
    }
}

pub struct ObserverSystems {
    executor: Box<dyn Fn(Blob, &Blob, &World) + Send + Sync>,
    systems: Blob,
    priority: u32,
}

impl ObserverSystems {
    pub fn new<A: Action>() -> Self {
        Self {
            executor: Box::new(move |mut outputs, systems, world| {
                let outputs = outputs.to_vec();

                for system in systems.iter_mut::<Observer<A>>() {
                    system.run(&outputs, world);
                }
            }),
            systems: Blob::new::<Observer<A>>(),
            priority: A::PRIORITY,
        }
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn add_observer<A: Action>(&mut self, observer: Observer<A>) {
        self.systems.push(observer);
    }

    pub fn add_observers<A: Action>(&mut self, observers: &mut Vec<Observer<A>>) {
        self.systems.extend(observers);
    }

    pub fn execute(&mut self, outputs: Blob, world: &World) {
        (self.executor)(outputs, &self.systems, world);
    }
}

#[derive(Default)]
pub struct Observables {
    observers: SparseMap<TypeId, ObserverSystems>,
}

impl Observables {
    pub fn new() -> Self {
        Self {
            observers: SparseMap::new(),
        }
    }

    pub fn add_observer<A: Action>(&mut self, observer: Observer<A>) {
        let type_id = TypeId::of::<A>();

        if let Some(systems) = self.observers.get_mut(&type_id) {
            systems.add_observer(observer);
        } else {
            let mut systems = ObserverSystems::new::<A>();
            systems.add_observer(observer);
            self.observers.insert(type_id, systems);
        }

        self.sort();
    }

    pub fn add_observers<A: Action>(&mut self, mut observers: Observers<A>) {
        let type_id = TypeId::of::<A>();

        if let Some(systems) = self.observers.get_mut(&type_id) {
            systems.add_observers(&mut observers.take());
        } else {
            let mut systems = ObserverSystems::new::<A>();
            systems.add_observers(&mut observers.take());
            self.observers.insert(type_id, systems);
        }

        self.sort();
    }

    pub fn sort(&mut self) {
        self.observers
            .sort(|(_, a), (_, b)| a.priority().cmp(&b.priority()));
    }

    pub fn execute(&mut self, mut outputs: ActionOutputs, world: &World) {
        for (type_id, observers) in self.observers.iter_mut() {
            if let Some(outputs) = outputs.remove(type_id) {
                observers.execute(outputs, world);
            }
        }
    }
}

impl Resource for Observables {}

macro_rules! impl_into_observer {
    ($($arg:ident),*) => {
        impl<Act: Action, F, $($arg: SystemArg),*> IntoObserver<Act, (F, $($arg),*)> for F
        where
            for<'a> F: Fn(&[Act::Output], $($arg),*) + Fn(&[Act::Output], $(ArgItem<'a, $arg>),*) + 'static,
        {
            fn into_observer(self) -> Observer<Act> {
                let mut reads = vec![];
                let mut writes = vec![];
                let mut metas = vec![];

                $(metas.extend($arg::metas());)*

                AccessMeta::collect(&mut reads, &mut writes, &metas);

                let system = Observer::<Act>::new(move |outputs: &[Act::Output], world: &World| {
                    (self)(outputs, $($arg::get(world)),*);
                }, reads, writes);

                system
            }
        }
    };
}

impl_into_observer!(A);
impl_into_observer!(A, B);
impl_into_observer!(A, B, C);
impl_into_observer!(A, B, C, D);
impl_into_observer!(A, B, C, D, E);
impl_into_observer!(A, B, C, D, E, F2);
