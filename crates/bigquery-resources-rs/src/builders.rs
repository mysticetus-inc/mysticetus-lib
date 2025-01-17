pub mod table_field_schema;

/// Marker type for an unset value in a builder.
///
/// Serves as a slightly more clear alternative to [`()`]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Unset;
