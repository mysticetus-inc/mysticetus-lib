use crate::Reference;

pub struct ChangeMap {
    map: fxhash::FxHashMap<Box<Reference>, Change>,
}

enum Change {}
