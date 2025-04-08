use std::cmp::Ordering;

use protos::firestore::Document;
use protos::firestore::structured_query::{Direction, Order};

mod change_map;
mod listener;
pub use listener::Listener;

pub trait ListenerType {
    fn cmp_documents(&self, a: &Document, b: &Document) -> Ordering;
}

struct QueryListener {
    orderings: Vec<Order>,
}

impl ListenerType for QueryListener {
    fn cmp_documents(&self, a: &Document, b: &Document) -> Ordering {
        let mut last_direction = Direction::Ascending;

        for ordering in self.orderings.iter() {
            let Some(ref field) = ordering.field else {
                continue;
            };

            last_direction = ordering.direction();

            let cmp = if field.field_path == "__name__" {
                a.name.cmp(&b.name)
            } else {
                let a_value = crate::util::extract_value(&a.fields, &field.field_path);
                let b_value = crate::util::extract_value(&a.fields, &field.field_path);
            };
        }

        todo!()
    }
}
