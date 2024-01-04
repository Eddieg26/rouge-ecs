use crate::storage::blob::Ptr;

pub trait BaseQuery: 'static {}

pub trait BaseFetch: 'static {}

pub trait Fetch: 'static {}

pub struct Query<'a, Q: BaseQuery> {
    ptrs: Vec<Ptr<'a>>,
    _marker: std::marker::PhantomData<Q>,
}
