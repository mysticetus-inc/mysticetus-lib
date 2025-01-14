use std::sync::Arc;

#[derive(Default)]
struct State {
    project_id: Option<Arc<str>>,
}
