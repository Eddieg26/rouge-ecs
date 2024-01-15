use std::any::TypeId;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Access {
    Read,
    Write,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccessMeta {
    ty: TypeId,
    access: Access,
}

impl AccessMeta {
    pub fn new<T: 'static>(access: Access) -> Self {
        Self {
            ty: TypeId::of::<T>(),
            access,
        }
    }

    pub fn ty(&self) -> TypeId {
        self.ty
    }

    pub fn access(&self) -> Access {
        self.access
    }

    pub fn collect(reads: &mut Vec<TypeId>, writes: &mut Vec<TypeId>, metas: &[AccessMeta]) {
        for meta in metas {
            match meta.access() {
                Access::Read => reads.push(meta.ty()),
                Access::Write => writes.push(meta.ty()),
            }
        }
    }
}
