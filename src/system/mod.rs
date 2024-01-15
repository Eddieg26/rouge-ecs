use crate::{
    core::Entities,
    world::{
        meta::{Access, AccessMeta},
        resource::Resource,
        World,
    },
};
use std::any::TypeId;

pub mod action;
pub mod observers;

pub struct System {
    function: Box<dyn for<'a> Fn(&'a World)>,
    reads: Vec<TypeId>,
    writes: Vec<TypeId>,
}

impl System {
    fn new<F: for<'a> Fn(&'a World) + 'static>(function: F) -> Self {
        Self {
            function: Box::new(function),
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    pub fn reads(&self) -> &[TypeId] {
        &self.reads
    }

    pub fn writes(&self) -> &[TypeId] {
        &self.writes
    }

    fn set_reads(&mut self, reads: Vec<TypeId>) {
        self.reads = reads;
    }

    fn set_writes(&mut self, writes: Vec<TypeId>) {
        self.writes = writes;
    }

    pub fn run(&self, world: &World) {
        (self.function)(world);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunMode {
    Parallel,
    Exclusive,
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
            for<'a> F: Fn($($arg),*) + Fn($(ArgItem<'a, $arg>),*) + 'static,
        {
            fn into_system(self) -> System {
                let mut reads = vec![];
                let mut writes = vec![];
                let mut metas = vec![];

                $(metas.extend($arg::metas());)*

                AccessMeta::collect(&mut reads, &mut writes, &metas);

                let mut system = System::new(move |world| {
                    (self)($($arg::get(world)),*);
                });

                system.set_reads(reads);
                system.set_writes(writes);

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
