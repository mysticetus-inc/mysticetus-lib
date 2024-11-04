
use sharded_slab::{Clear, Pool};

pub type Lol = ();

struct ActiveSpans {
    stack: smallvec::SmallVec<[(); 16]>,
}

pub struct Registry {
    spans: Pool<Inner>,
}

#[derive(Default)]
struct Inner {}

impl Clear for Inner {
    fn clear(&mut self) {}
}
