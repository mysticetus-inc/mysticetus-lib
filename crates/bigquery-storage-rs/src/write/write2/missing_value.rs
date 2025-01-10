use std::collections::HashMap;

use protos::bigquery_storage::append_rows_request::MissingValueInterpretation;

#[derive(Debug, Clone, Default)]
pub struct MissingValueInterpretations {
    pub(crate) per_field: HashMap<String, i32>,
    pub(crate) default: MissingValueInterpretation,
}

impl MissingValueInterpretations {
    pub(super) fn pair_from_opt(
        opt: &Option<Self>,
    ) -> (HashMap<String, i32>, MissingValueInterpretation) {
        match opt {
            Some(Self { per_field, default }) => (per_field.clone(), default.clone()),
            None => (HashMap::new(), MissingValueInterpretation::default()),
        }
    }
}
