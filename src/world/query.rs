use super::{
    meta::{Access, AccessMeta},
    World,
};
use crate::{
    archetype::ArchetypeId,
    core::{Component, ComponentId, Entity},
    storage::table::Table,
    system::SystemArg,
};

pub trait BaseQuery {
    type Item<'a>;

    fn init(_: &World, _: &mut QueryState) {}
    fn fetch(world: &World, entity: Entity) -> Self::Item<'_>;
    fn metas() -> Vec<AccessMeta>;
}

impl<C: Component> BaseQuery for &C {
    type Item<'a> = &'a C;

    fn init(world: &World, state: &mut QueryState) {
        state.add_component(world.component_id::<C>());
    }

    fn fetch(world: &World, entity: Entity) -> Self::Item<'_> {
        world.component::<C>(entity).unwrap()
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<C>(Access::Read)]
    }
}

impl<C: Component> BaseQuery for &mut C {
    type Item<'a> = &'a mut C;

    fn init(world: &World, state: &mut QueryState) {
        state.add_component(world.component_id::<C>());
    }

    fn fetch(world: &World, entity: Entity) -> Self::Item<'_> {
        world.component_mut::<C>(entity).unwrap()
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<C>(Access::Write)]
    }
}

impl<C: Component> BaseQuery for Option<&C> {
    type Item<'a> = Option<&'a C>;

    fn fetch(world: &World, entity: Entity) -> Self::Item<'_> {
        world.component::<C>(entity)
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<C>(Access::Read)]
    }
}

impl<C: Component> BaseQuery for Option<&mut C> {
    type Item<'a> = Option<&'a mut C>;

    fn fetch(world: &World, entity: Entity) -> Self::Item<'_> {
        world.component_mut::<C>(entity)
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<C>(Access::Write)]
    }
}

impl BaseQuery for Entity {
    type Item<'a> = Entity;

    fn fetch(_world: &World, entity: Entity) -> Self::Item<'_> {
        entity
    }

    fn metas() -> Vec<AccessMeta> {
        vec![AccessMeta::new::<Entity>(Access::Read)]
    }
}

pub struct Query<'a, Q: BaseQuery> {
    world: &'a World,
    tables: Box<[&'a Table<Entity>]>,
    table_index: usize,
    row_index: usize,
    _marker: std::marker::PhantomData<Q>,
}

impl<'a, Q: BaseQuery> Query<'a, Q> {
    pub fn new(world: &'a World) -> Self {
        let mut state = QueryState::new();
        Q::init(world, &mut state);
        let tables = world
            .archetypes()
            .archetypes(state.components(), &[])
            .iter()
            .map(|id| ArchetypeId::into(**id))
            .collect::<Vec<_>>();
        let tables = world.tables().array(&tables);

        Self {
            world,
            tables,
            table_index: 0,
            row_index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct QueryState {
    components: Vec<ComponentId>,
}

impl QueryState {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn add_component(&mut self, component: ComponentId) {
        self.components.push(component);
    }

    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }
}

impl<'a, Q: BaseQuery> Iterator for Query<'a, Q> {
    type Item = Q::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.table_index >= self.tables.len() {
            return None;
        } else if self.row_index >= self.tables[self.table_index].len() {
            self.table_index += 1;
            self.row_index = 0;
            return self.next();
        } else {
            let entity = self.tables[self.table_index].rows()[self.row_index];
            self.row_index += 1;

            Some(Q::fetch(self.world, entity))
        }
    }
}

impl<Q: BaseQuery> SystemArg for Query<'_, Q> {
    type Item<'a> = Query<'a, Q>;

    fn get<'a>(world: &'a World) -> Self::Item<'a> {
        Query::new(world)
    }

    fn metas() -> Vec<AccessMeta> {
        Q::metas()
    }
}

macro_rules! impl_base_query_for_tuples {
    ($(($($name:ident),+)),+) => {
        $(
            impl<$($name: BaseQuery),+> BaseQuery for ($($name,)+) {
                type Item<'a> = ($($name::Item<'a>,)+);

                fn init(world: &World, state: &mut QueryState) {
                    $(
                        $name::init(world, state);
                    )+
                }

                fn fetch(world: &World, entity: Entity) -> Self::Item<'_> {
                    ($($name::fetch(world, entity),)+)
                }

                fn metas() -> Vec<AccessMeta> {
                    let mut metas = Vec::new();
                    $(
                        metas.extend($name::metas());
                    )+
                    metas
                }
            }
        )+
    };
}

impl_base_query_for_tuples!((A, B));
impl_base_query_for_tuples!((A, B, C));
impl_base_query_for_tuples!((A, B, C, D));
impl_base_query_for_tuples!((A, B, C, D, E));
impl_base_query_for_tuples!((A, B, C, D, E, F));
impl_base_query_for_tuples!((A, B, C, D, E, F, G));
impl_base_query_for_tuples!((A, B, C, D, E, F, G, H));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V));
// impl_base_query_for_tuples!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W));
// impl_base_query_for_tuples!((
//     A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
// ));
// impl_base_query_for_tuples!((
//     A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
// ));
// impl_base_query_for_tuples!((
//     A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
// ));
