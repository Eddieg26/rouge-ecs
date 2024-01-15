use super::{ArgItem, SystemArg};
use crate::world::{actions::Action, meta::AccessMeta, World};
use std::any::TypeId;

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

    pub(crate) fn take(&mut self) -> Vec<Observer<A>> {
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
