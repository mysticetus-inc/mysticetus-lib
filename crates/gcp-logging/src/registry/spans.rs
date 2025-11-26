use std::cell::RefCell;

use tracing::span::Id;

thread_local! {
    static SPANS: RefCell<Option<Vec<Id>>> = RefCell::new(None);
}

fn with_ref<O>(f: impl FnOnce(&Vec<Id>) -> O) -> Option<O> {
    SPANS.with_borrow(|spans| spans.as_ref().map(f))
}

fn with_mut<O>(f: impl FnOnce(&mut Vec<Id>) -> O) -> O {
    SPANS.with_borrow_mut(|spans| f(spans.get_or_insert_with(|| Vec::with_capacity(8))))
}

pub fn span_stack_len() -> usize {
    with_ref(|spans| spans.len()).unwrap_or(0)
}

pub trait SpanVisitor {
    type Output;

    fn visit_span(&mut self, span: &Id) -> std::ops::ControlFlow<Self::Output>;
}

impl<F, O> SpanVisitor for F
where
    for<'a> F: FnMut(&'a Id) -> std::ops::ControlFlow<O>,
{
    type Output = O;
    #[inline]
    fn visit_span(&mut self, span: &Id) -> std::ops::ControlFlow<Self::Output> {
        (self)(span)
    }
}

pub fn visit<S: SpanVisitor>(mut visitor: S) -> Option<S::Output> {
    with_ref(|spans| {
        for span in spans {
            match visitor.visit_span(span) {
                std::ops::ControlFlow::Break(out) => return Some(out),
                std::ops::ControlFlow::Continue(_) => (),
            }
        }

        None
    })
    .flatten()
}

pub fn visit_slice<O>(visitor: impl FnOnce(&[Id]) -> O) -> O {
    SPANS.with_borrow(|spans| {
        let span_slice = spans.as_deref().unwrap_or_default();
        visitor(span_slice)
    })
}

/// Gets the current span.
pub fn current() -> Option<Id> {
    with_ref(|spans| spans.last().cloned()).flatten()
}

/// Enters the given span, making it the current span. Returns true
/// if this is the first instance of this span being active in this TLS stack.
pub fn enter(id: &Id) -> bool {
    with_mut(|spans| {
        let duplicate = spans.iter().any(|existing| existing == id);
        spans.push(id.clone());
        !duplicate
    })
}

/// Exits the span given by 'id'. Returns true if it was found and removed,
/// false if the span didn't exist, or wasn't currently entered.
pub fn exit(id: &Id) -> bool {
    with_mut(|spans| {
        let mut rev_iter = spans.iter().enumerate().rev();

        if let Some((idx, _)) = rev_iter.by_ref().find(|(_, i)| *i == id) {
            let duplicate = rev_iter.any(|(_, i)| i == id);
            spans.remove(idx);
            !duplicate
        } else {
            false
        }
    })
}
