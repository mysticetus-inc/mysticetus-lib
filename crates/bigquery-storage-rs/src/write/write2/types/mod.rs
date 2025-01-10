mod default;

pub use default::DefaultStream;

pub trait StreamType: private::SealedStreamType {}

pub(super) type No = typenum::B0;
pub(super) type Yes = typenum::B1;

pub(super) trait Boolean: typenum::Bit {
    const VALUE: bool;
}

impl Boolean for No {
    const VALUE: bool = false;
}

impl Boolean for Yes {
    const VALUE: bool = true;
}

impl<T: private::SealedStreamType> StreamType for T {}

mod private {
    use std::borrow::Cow;

    use protos::bigquery_storage::{write_stream, AppendRowsResponse};

    use super::Boolean;

    pub trait SealedStreamType {
        const WRITE_STREM_TYPE: write_stream::Type;

        type NeedsFlush: Boolean;
        type NeedsCommit: Boolean;
        type NeedsFinalize: Boolean;
        type OffsetAllowed: Boolean;

        /// Fully qualified table, in the form:
        /// `projects/{project}/datasets/{dataset}/tables/{table}`
        fn parent(&self) -> Cow<'_, str>;

        /// Fully qualified write stream name, in the form:
        /// `projects/{project}/datasets/{dataset}/tables/{table}/streams/{stream_name}`
        fn write_stream(&self) -> Cow<'_, str>;

        fn process_result(&self, result: &AppendRowsResponse) -> crate::Result<()>;

        #[inline]
        fn offset(&self) -> Option<i64> {
            None
        }

        #[inline]
        fn update_offset(&mut self, offset: i64) {
            let _ = offset;
        }
    }
}
