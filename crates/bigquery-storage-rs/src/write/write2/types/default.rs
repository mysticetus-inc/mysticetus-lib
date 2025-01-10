use std::borrow::Cow;

use protos::bigquery_storage::write_stream;

#[derive(Debug, Clone)]
pub struct DefaultStream {
    parent_cutoff_idx: usize,
    write_stream_name: String,
}

impl DefaultStream {
    pub fn new(project_id: &str, dataset_id: &str, table_id: &str) -> Self {
        let mut write_stream_name = String::with_capacity(
            "projects/".len()
                + project_id.len()
                + "/datasets/".len()
                + dataset_id.len()
                + "/tables/".len()
                + table_id.len()
                + "/streams/_default".len(),
        );

        write_stream_name.push_str("projects/");
        write_stream_name.push_str(project_id);
        write_stream_name.push_str("/datasets/");
        write_stream_name.push_str(dataset_id);
        write_stream_name.push_str("/tables/");
        write_stream_name.push_str(table_id);
        let parent_cutoff_idx = write_stream_name.len() + 1;
        write_stream_name.push_str("/streams/_default");

        Self {
            parent_cutoff_idx,
            write_stream_name,
        }
    }

    pub fn change_table(&mut self, new_table: &str) {
        self.write_stream_name.truncate(self.parent_cutoff_idx);
        self.write_stream_name.pop();
        while self.write_stream_name.pop() != Some('/') {}

        self.write_stream_name
            .reserve(new_table.len() + "/streams/_default".len());
        self.write_stream_name.push_str(new_table);
        self.parent_cutoff_idx = self.write_stream_name.len() + 1;
        self.write_stream_name.push_str("/streams/_default");
    }
}

impl super::private::SealedStreamType for DefaultStream {
    const WRITE_STREM_TYPE: write_stream::Type = write_stream::Type::Committed;

    type NeedsFinalize = super::No;
    type NeedsFlush = super::No;
    type NeedsCommit = super::No;
    type OffsetAllowed = super::No;

    #[inline]
    fn parent(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.write_stream_name[..self.parent_cutoff_idx])
    }

    #[inline]
    fn write_stream(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.write_stream_name)
    }

    #[inline]
    fn process_result(
        &self,
        _result: &protos::bigquery_storage::AppendRowsResponse,
    ) -> crate::Result<()> {
        Ok(())
    }
}
