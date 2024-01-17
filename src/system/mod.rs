use crate::{
    core::Entities,
    world::{
        meta::{Access, AccessMeta},
        resource::Resource,
        World,
    },
};
use std::any::TypeId;

pub mod observer;

pub struct System {
    function: Box<dyn for<'a> Fn(&'a World) + Send + Sync>,
    reads: Vec<TypeId>,
    writes: Vec<TypeId>,
}

impl System {
    fn new<F>(function: F, reads: Vec<TypeId>, writes: Vec<TypeId>) -> Self
    where
        F: for<'a> Fn(&'a World) + Send + Sync + 'static,
    {
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

    pub fn run(&self, world: &World) {
        (self.function)(world);
    }
}

/// A collection of systems that can be run in sequence.
pub struct SystemSet {
    systems: Vec<System>,
}

impl SystemSet {
    pub fn new() -> Self {
        Self { systems: vec![] }
    }

    pub fn add_system<M>(&mut self, system: impl IntoSystem<M>) {
        self.systems.push(system.into_system());
    }

    pub fn append(&mut self, mut system_set: SystemSet) {
        self.systems.append(&mut system_set.systems);
    }

    pub fn reads(&self) -> Vec<TypeId> {
        self.systems
            .iter()
            .flat_map(|system| system.reads().to_vec())
            .collect()
    }

    pub fn writes(&self) -> Vec<TypeId> {
        self.systems
            .iter()
            .flat_map(|system| system.writes().to_vec())
            .collect()
    }
}

impl IntoSystem<()> for SystemSet {
    fn into_system(self) -> System {
        let mut reads = vec![];
        let mut writes = vec![];

        for system in &self.systems {
            reads.extend(system.reads().to_vec());
            writes.extend(system.writes().to_vec());
        }

        let system = System::new(
            move |world| {
                for system in &self.systems {
                    system.run(world);
                }
            },
            reads,
            writes,
        );

        system
    }
}

pub trait SystemArg {
    type Item<'a>;

    fn get<'a>(world: &'a World) -> Self::Item<'a>;
    fn metas() -> Vec<AccessMeta>;
}

impl SystemArg for &World {
    type Item<'a> = &'a World;

    fn get<'a>(world: &'a World) -> Self::Item<'a> {
        world
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<World>(Access::Read)]
    }
}

pub type ArgItem<'a, A> = <A as SystemArg>::Item<'a>;

pub trait IntoSystem<M> {
    fn into_system(self) -> System;
}

pub trait IntoSystems<M> {
    fn into_systems(self) -> Vec<System>;
}

impl<R: Resource> SystemArg for &R {
    type Item<'a> = &'a R;

    fn get<'a>(world: &'a World) -> Self::Item<'a> {
        world.resource::<R>()
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<R>(Access::Read)]
    }
}

impl<R: Resource> SystemArg for &mut R {
    type Item<'a> = &'a mut R;

    fn get<'a>(world: &'a World) -> Self::Item<'a> {
        world.resource_mut::<R>()
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<R>(Access::Write)]
    }
}

impl SystemArg for &Entities {
    type Item<'a> = &'a Entities;

    fn get<'a>(world: &'a World) -> Self::Item<'a> {
        world.entities()
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<Entities>(Access::Read)]
    }
}

macro_rules! impl_into_system {
    ($($arg:ident),*) => {
        impl<F, $($arg: SystemArg),*> IntoSystem<(F, $($arg),*)> for F
        where
            for<'a> F: Fn($($arg),*) + Fn($(ArgItem<'a, $arg>),*) + Send + Sync + 'static,
        {
            fn into_system(self) -> System {
                let mut reads = vec![];
                let mut writes = vec![];
                let mut metas = vec![];

                $(metas.extend($arg::metas());)*

                AccessMeta::pick(&mut reads, &mut writes, &metas);

                let system = System::new(move |world| {
                    (self)($($arg::get(world)),*);
                }, reads, writes);

                system
            }
        }

        impl<$($arg: SystemArg),*> SystemArg for ($($arg,)*) {
            type Item<'a> = ($($arg::Item<'a>,)*);

            fn get<'a>(world: &'a World) -> Self::Item<'a> {
                ($($arg::get(world),)*)
            }

            fn metas() -> Vec<AccessMeta> {
                let mut metas = Vec::new();
                $(metas.extend($arg::metas());)*
                metas
            }
        }
    };
}

impl_into_system!(A);
impl_into_system!(A, B);
impl_into_system!(A, B, C);
impl_into_system!(A, B, C, D);
impl_into_system!(A, B, C, D, E);
impl_into_system!(A, B, C, D, E, F2);
impl_into_system!(A, B, C, D, E, F2, G);
impl_into_system!(A, B, C, D, E, F2, G, H);
impl_into_system!(A, B, C, D, E, F2, G, H, I);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
// impl_into_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
