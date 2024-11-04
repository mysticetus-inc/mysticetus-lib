use std::cmp::Ordering;

use protos::bigquery_storage::table_field_schema::{Mode, Type as FieldType};
use protos::bigquery_storage::{TableFieldSchema, TableSchema};
use protos::protobuf::field_descriptor_proto::{Label, Type as ProtoType};
use protos::protobuf::{DescriptorProto, FieldDescriptorProto};

use super::encode::{Field, WireType};

macro_rules! build_int_enum_pairs {
    ($($e:expr),* $(,)?) => {{
        sort_pairs([ $($e as i32, $e),* ])
    }};

    ($(
        $e:expr => ($($paired:expr),* $(,)?)
    ),* $(,)?) => {{
        sort_pairs([
            $(
            ($e as i32, ($e, $($paired),*))
            ),*
        ])
    }};
}

const TYPE_MAPPING: [(i32, (FieldType, ProtoType, WireType)); 15] = build_int_enum_pairs! {
    FieldType::Datetime => (ProtoType::String, WireType::LengthDelimited),
    FieldType::Geography => (ProtoType::String, WireType::LengthDelimited),
    FieldType::Time => (ProtoType::String, WireType::LengthDelimited),
    FieldType::Json => (ProtoType::String, WireType::LengthDelimited),
    FieldType::Numeric => (ProtoType::String, WireType::LengthDelimited),
    FieldType::Bignumeric => (ProtoType::String, WireType::LengthDelimited),
    FieldType::String => (ProtoType::String, WireType::LengthDelimited),
    FieldType::Int64 => (ProtoType::Sint64, WireType::Varint),
    FieldType::Double => (ProtoType::Double, WireType::Bits64),
    FieldType::Struct => (ProtoType::Message, WireType::LengthDelimited),
    FieldType::Timestamp => (ProtoType::Sfixed64, WireType::Bits64),
    FieldType::Bytes => (ProtoType::Bytes, WireType::LengthDelimited),
    FieldType::Bool => (ProtoType::Bool, WireType::Varint),
    FieldType::Date => (ProtoType::Int32, WireType::Varint),
    FieldType::Interval => (ProtoType::Int64, WireType::LengthDelimited),
};

const MODE_MAPPING: [(i32, (Mode, Label)); 3] = build_int_enum_pairs! {
    Mode::Nullable => (Label::Optional),
    Mode::Required => (Label::Required),
    Mode::Repeated => (Label::Repeated),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Required {
    No = 0,
    Yes = 1,
}

impl Required {
    const fn from_mode_int(mode: i32) -> Self {
        if mode == Mode::Required as i32 {
            Self::Yes
        } else {
            Self::No
        }
    }
}

#[derive(Debug)]
pub struct Schemas {
    #[allow(unused)]
    table: TableSchema,
    proto: DescriptorProto,
    field_index_map: Vec<FieldIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldIndex {
    field_name: String,
    proto_field: Field,
    required: Required,
}

impl FieldIndex {
    #[inline]
    pub const fn is_required(&self) -> bool {
        matches!(self.required, Required::Yes)
    }

    #[inline]
    pub fn encode_tag<B>(&self, buf: &mut B)
    where
        B: bytes::BufMut,
    {
        buf.put_u8(self.proto_field.to_byte());
    }

    #[inline]
    pub const fn wire_type(&self) -> WireType {
        self.proto_field.wire_type()
    }

    #[inline]
    fn cmp_field_name(&self, name: &str) -> Ordering {
        let self_iter = self.field_name.chars().map(char::to_lowercase).flatten();
        let name_iter = name.chars().map(char::to_lowercase).flatten();

        self_iter.cmp(name_iter)
    }
}

impl Schemas {
    fn new_inner(name: Option<String>, table: TableSchema) -> Result<Self, super::EncodeError> {
        let mut fields = Vec::with_capacity(table.fields.len());

        let mut field_index_map = Vec::with_capacity(table.fields.len());

        for (idx, table_field) in table.fields.iter().enumerate() {
            let (proto_field, wire_type) = table_field_to_field_descriptor(table_field, idx + 1);
            fields.push(proto_field);

            let wire_type = wire_type.ok_or_else(|| {
                super::EncodeError::Misc(format!(
                    "could not determine wire type for table field: {:#?}",
                    table_field,
                ))
            })?;

            field_index_map.push(FieldIndex {
                field_name: table_field.name.clone(),
                proto_field: Field::new((idx + 1) as u8, wire_type),
                required: Required::from_mode_int(table_field.mode),
            });
        }

        let proto = DescriptorProto {
            name,
            field: fields,
            ..Default::default()
        };

        field_index_map.sort();

        Ok(Self {
            table,
            proto,
            field_index_map,
        })
    }

    pub fn new_with_type_name<T>(table: TableSchema) -> Result<Self, super::EncodeError> {
        let name = proto_name::from_type::<T>();
        Self::new_inner(Some(name.into_owned()), table)
    }

    pub fn new_with_name<N>(name: N, table: TableSchema) -> Result<Self, super::EncodeError>
    where
        N: AsRef<str>,
    {
        let name = proto_name::extract_safe_name(name.as_ref());
        Self::new_inner(Some(name.into_owned()), table)
    }

    #[inline]
    pub const fn proto(&self) -> &DescriptorProto {
        &self.proto
    }

    pub fn get_field_index<S>(&self, field: S) -> Option<&FieldIndex>
    where
        S: AsRef<str>,
    {
        self.field_index_map
            .binary_search_by(|field_idx| field_idx.cmp_field_name(field.as_ref()))
            .map(|idx| &self.field_index_map[idx])
            .ok()
    }
}

pub mod proto_name {
    const FALLBACK_NAME: &str = "FallbackProtoName";

    use std::borrow::Cow;
    pub fn from_type<T>() -> Cow<'static, str> {
        extract_safe_name(std::any::type_name::<T>())
    }

    pub fn extract_safe_name<'a>(s: &'a str) -> Cow<'a, str> {
        let mut type_name = s;
        // iterate over separated paths
        for component in s.split("::") {
            type_name = component;

            // the first component with generics will contain our type name, otherwise
            // the final component in the iterator will be the type name.
            if component.contains('<') {
                break;
            }
        }

        type_name = type_name.trim();

        if let Some(generic_start_idx) = type_name.find('<') {
            type_name = &type_name[..generic_start_idx];
        }

        // double check we have no non-ascii alphanumeric characters, remove if so
        let name = if type_name
            .find(|ch: char| !ch.is_ascii_alphanumeric())
            .is_some()
        {
            Cow::Owned(type_name.replace(|ch: char| !ch.is_ascii_alphanumeric(), ""))
        } else {
            Cow::Borrowed(type_name)
        };

        // will likely never happen but, making sure there's a fallback is never a bad call.
        if name.is_empty() {
            Cow::Borrowed(FALLBACK_NAME)
        } else {
            name
        }
    }

    #[test]
    fn test_generated_proto_names() {
        const TESTS: &[(&str, &[&str])] = &[
            ("Option", &[
                "std::option::Option<std::string::String>",
                "Option<T>",
                "::std::option::Option<Inner<InnerInner>>",
                "Option<&'a T>",
            ]),
            ("TestStruct", &["inner::TestStruct"]),
            (
                // check that the fallback works for empty strings and weird values
                FALLBACK_NAME,
                &["", "::", "::<>::<>::<>()"],
            ),
        ];

        for (expected, inner_tests) in TESTS.iter() {
            for test in inner_tests.iter() {
                assert_eq!(*expected, extract_safe_name(test));
            }
        }
    }
}

fn get_proto_type_and_mode(
    field: &TableFieldSchema,
) -> (Option<ProtoType>, Option<WireType>, Option<Label>) {
    let (types, wire_type) = TYPE_MAPPING
        .binary_search_by_key(&field.r#type, |(type_int, _)| *type_int)
        .map(|idx| {
            let elem = TYPE_MAPPING[idx];
            (elem.1.1, elem.1.2)
        })
        .ok()
        .unzip();

    let mode = MODE_MAPPING
        .binary_search_by_key(&field.mode, |(mode_int, _)| *mode_int)
        .map(|idx| MODE_MAPPING[idx].1.1)
        .ok();

    (types, wire_type, mode)
}

fn table_field_to_field_descriptor(
    field: &TableFieldSchema,
    field_index: usize,
) -> (FieldDescriptorProto, Option<WireType>) {
    let (proto_type, wire_type, mode) = get_proto_type_and_mode(field);

    macro_rules! cast_opt_to_i32 {
        ($opt_enum:expr) => {{
            match $opt_enum {
                Some(inner) => Some(inner as i32),
                None => None,
            }
        }};
    }

    let field_descrip = FieldDescriptorProto {
        name: Some(field.name.clone()),
        number: Some(field_index as i32),
        label: cast_opt_to_i32!(mode),
        r#type: cast_opt_to_i32!(proto_type),
        type_name: None,
        extendee: None,
        default_value: None,
        oneof_index: None,
        json_name: None,
        options: None,
        proto3_optional: None,
    };

    (field_descrip, wire_type)
}

/// ----- Helper functions for the macros defined at the top -------- ///

#[test]
fn ensure_sorted() {
    assert!(MODE_MAPPING.is_sorted(), "MODE_MAPPING not sorted!");
    assert!(TYPE_MAPPING.is_sorted(), "TYPE_MAPPING not sorted!");
}

/// Since [`[T]::sort`] isn't const stable, this simple bubble sort will suffice.
///
/// Performs 1 pass, sifting larger elements towards the end. Returns true if no elements were
/// moved (i.e the slice is sorted).
const fn bubble_sort_pass<T>(slice: &mut [(i32, T)]) -> bool {
    let mut num_swapped = 0;

    let mut idx = 0;

    while idx < slice.len() - 1 {
        let curr_enum_variant = slice[idx].0;
        let next_enum_variant = slice[idx + 1].0;

        if curr_enum_variant == next_enum_variant {
            panic!("repeated enum value found");
        }

        if next_enum_variant < curr_enum_variant {
            slice.swap(idx, idx + 1);
            num_swapped += 1;
        }

        idx += 1;
    }

    num_swapped == 0
}

const fn sort_pairs<const N: usize, T>(mut src: [(i32, T); N]) -> [(i32, T); N] {
    while !bubble_sort_pass(&mut src) {}

    src
}
