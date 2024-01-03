use storage::blob::Blob;

pub mod core;
pub mod storage;
pub mod world;

fn main() {
    let mut blob = Blob::new::<u32>();
    blob.push(8);
    blob.push(50);
    blob.push(0);
    blob.push(17);
    blob.push(39);

    // for index in 0..blob.len() {
    //     println!("{:?}", blob.get::<u32>(index));
    // }

    // let ptrs: Vec<Ptr>;

    let ptr = blob.ptr();

    for index in 0..blob.len() {
        println!("{:?}", ptr.get::<u32>(index));
    }
}

pub trait BaseQuery: 'static {}

pub struct Query<Q: BaseQuery> {
    ptrs: Vec<Ptr>,
    _marker: std::marker::PhantomData<Q>,
}
