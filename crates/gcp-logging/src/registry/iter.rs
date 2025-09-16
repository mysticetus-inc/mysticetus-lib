use crate::registry::{DataRef, Records};

pub struct SpanDataIter<'a> {
    pub(super) records: &'a Records,
    pub(super) next_id: Option<tracing::Id>,
}

impl<'a> Iterator for SpanDataIter<'a> {
    type Item = DataRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.next_id.take()?;
        let data_ref = self.records.get(&id)?;
        self.next_id = data_ref.parent.clone();
        Some(data_ref)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.next_id.is_none() {
            (0, Some(0))
        } else {
            (0, None)
        }
    }
}
