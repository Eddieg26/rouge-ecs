use super::{ArgItem, SystemArg};
use crate::world::{actions::Action, meta::AccessMeta, World};
use std::any::TypeId;

pub struct ActionSystem<A: Action> {
    function: Box<dyn Fn(&[A::Output], &World)>,
    reads: Vec<TypeId>,
    writes: Vec<TypeId>,
}

impl<A: Action> ActionSystem<A> {
    fn new(function: impl Fn(&[A::Output], &World) + 'static) -> Self {
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

    pub fn run(&mut self, outputs: &[A::Output], world: &World) {
        (self.function)(outputs, world);
    }
}

pub struct ActionSystems<A: Action> {
    systems: Vec<ActionSystem<A>>,
}

impl<A: Action> ActionSystems<A> {
    pub fn new() -> Self {
        Self { systems: vec![] }
    }

    pub fn add_system<M>(mut self, system: impl IntoActionSystem<A, M>) -> Self {
        self.systems.push(system.into_system());

        self
    }

    pub(crate) fn take(&mut self) -> Vec<ActionSystem<A>> {
        std::mem::take(&mut self.systems)
    }
}

pub trait IntoActionSystem<A: Action, M> {
    fn into_system(self) -> ActionSystem<A>;
}

impl<A: Action, F> IntoActionSystem<A, F> for F
where
    F: Fn(&[A::Output]) + 'static,
{
    fn into_system(self) -> ActionSystem<A> {
        ActionSystem::new(move |outputs: &[A::Output], _: &World| {
            (self)(outputs);
        })
    }
}

macro_rules! impl_into_action_system {
    ($($arg:ident),*) => {
        impl<Act: Action, F, $($arg: SystemArg),*> IntoActionSystem<Act, (F, $($arg),*)> for F
        where
            for<'a> F: Fn(&[Act::Output], $($arg),*) + Fn(&[Act::Output], $(ArgItem<'a, $arg>),*) + 'static,
        {
            fn into_system(self) -> ActionSystem<Act> {
                let mut reads = vec![];
                let mut writes = vec![];
                let mut metas = vec![];

                $(metas.extend($arg::metas());)*

                AccessMeta::collect(&mut reads, &mut writes, &metas);

                let mut system = ActionSystem::<Act>::new(move |outputs: &[Act::Output], world: &World| {
                    (self)(outputs, $($arg::get(world)),*);
                });

                system.set_reads(reads);
                system.set_writes(writes);

                system
            }
        }
    };
}

impl_into_action_system!(A);
impl_into_action_system!(A, B);
impl_into_action_system!(A, B, C);
impl_into_action_system!(A, B, C, D);
impl_into_action_system!(A, B, C, D, E);
impl_into_action_system!(A, B, C, D, E, F2);
// impl_into_action_system!(A, B, C, D, E, F2, G);
// impl_into_action_system!(A, B, C, D, E, F2, G, H);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
// impl_into_action_system!(A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
// impl_into_action_system!(
//     A, B, C, D, E, F2, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
// );
