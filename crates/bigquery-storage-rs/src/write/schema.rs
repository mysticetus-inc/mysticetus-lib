use fxhash::FxBuildHasher;
use protos::protobuf::DescriptorProto;

use crate::proto::EncodeError;

mod field_info;
mod table_schema;
pub use field_info::FieldInfo;
pub use table_schema::{FieldSchema, TableSchema};

#[derive(Debug)]
pub struct Schema {
    fields: Box<[FieldInfo]>,
    index: fxhash::FxHashMap<Box<str>, usize>,
}

impl Schema {
    pub fn from_table_schema(schema: impl TableSchema) -> Result<Self, EncodeError> {
        let schema = schema.into_fields().into_iter();

        let schema_len = schema.len();

        assert!(
            schema_len <= u8::MAX as usize,
            "table with more than 255 fields"
        );

        let mut fields = Vec::with_capacity(schema_len);

        let mut index =
            fxhash::FxHashMap::with_capacity_and_hasher(schema_len, FxBuildHasher::default());

        for (idx, field) in schema.enumerate() {
            let field = FieldInfo::from_field(idx, field)?;

            if let Some(name) = index.insert(field.name().into(), idx) {
                panic!("duplicate field name: {name} ");
            }

            fields.push(field);
        }

        Ok(Self {
            index,
            fields: fields.into_boxed_slice(),
        })
    }

    pub fn to_descriptor_proto(&self) -> DescriptorProto {
        DescriptorProto {
            field: self.fields.iter().map(|field| field.to_proto()).collect(),
            ..Default::default()
        }
    }

    pub fn get_field_by_index(&self, index: usize) -> Option<&FieldInfo> {
        self.fields.get(index)
    }

    pub fn get_field<'a>(&'a self, field_name: &str) -> Option<&'a FieldInfo> {
        let index = self.index.get(field_name).copied()?;
        Some(&self.fields[index])
    }
}
