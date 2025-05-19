use std::fmt;

use bytes::BufMut;
use protos::bigquery_storage::table_field_schema::{Mode, Type as TableFieldType};
use protos::protobuf::FieldDescriptorProto;
use protos::protobuf::field_descriptor_proto::{Label, Type as FieldProtoType};

use super::FieldSchema;
use super::table_schema::proto_type_to_field_type;
use crate::proto::{EncodeError, Field, WireType};

#[derive(Clone, PartialEq, Eq)]
pub struct FieldInfo {
    name: Box<str>,
    packed: u16,
}

impl FieldInfo {
    fn from_parts(
        index: usize,
        wire_type: WireType,
        field_type: TableFieldType,
        mode: Mode,
        name: Box<str>,
    ) -> Self {
        let proto_field = Field::new(index as u8, wire_type);

        let packed = pack(proto_field, mode, field_type);

        Self { name, packed }
    }

    pub fn from_field(index: usize, field: impl FieldSchema) -> Result<Self, EncodeError> {
        let Some(field_type) = field.ty() else {
            return Err(EncodeError::UnspecifiedFieldType(
                field.into_field_name().into(),
            ));
        };

        let wire_type = crate::proto::field_type_to_wire_type(field_type);

        Ok(Self::from_parts(
            index,
            wire_type,
            field.proto_ty(),
            field.proto_mode(),
            field.into_field_name().into(),
        ))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn wire_type(&self) -> Result<WireType, EncodeError> {
        let field_type = proto_type_to_field_type(self.ty())
            .ok_or_else(|| EncodeError::UnspecifiedFieldType(self.name.clone()))?;

        Ok(crate::proto::field_type_to_wire_type(field_type))
    }

    pub fn encode_tag<B: ?Sized + BufMut>(&self, buf: &mut B) {
        buf.put_u8(self.packed.to_be_bytes()[0]);
    }

    pub fn field(&self) -> Field {
        Field::from_byte(self.packed.to_be_bytes()[0])
            .expect("this should always be valid once constructed")
    }

    pub fn index(&self) -> u8 {
        self.field().field_number()
    }

    pub fn mode(&self) -> Mode {
        let raw_mode = self.packed.to_be_bytes()[1] & 0b11;
        Mode::try_from(raw_mode as i32).unwrap_or(Mode::Unspecified)
    }

    pub fn ty(&self) -> TableFieldType {
        let raw_ty = self.packed.to_be_bytes()[1] >> 2;
        TableFieldType::try_from(raw_ty as i32).unwrap_or(TableFieldType::Unspecified)
    }

    pub(super) fn to_proto(&self) -> FieldDescriptorProto {
        let (field, mode, ty) = unpack(self.packed);

        FieldDescriptorProto {
            name: Some(self.name.as_ref().to_owned()),
            number: Some(field.field_number() as i32),
            label: match mode {
                Mode::Nullable => Some(Label::Optional as i32),
                Mode::Repeated => Some(Label::Repeated as i32),
                Mode::Required => Some(Label::Required as i32),
                Mode::Unspecified => None,
            },
            r#type: match ty {
                TableFieldType::Bool => Some(FieldProtoType::Bool as i32),
                TableFieldType::String => Some(FieldProtoType::String as i32),
                TableFieldType::Int64 => Some(FieldProtoType::Sint64 as i32),
                TableFieldType::Double => Some(FieldProtoType::Float as i32),
                TableFieldType::Struct => Some(FieldProtoType::Message as i32),
                TableFieldType::Bytes => Some(FieldProtoType::Bytes as i32),
                TableFieldType::Timestamp => Some(FieldProtoType::Int64 as i32),
                TableFieldType::Date => Some(FieldProtoType::Int32 as i32),
                TableFieldType::Time => Some(FieldProtoType::String as i32),
                TableFieldType::Datetime => Some(FieldProtoType::String as i32),
                TableFieldType::Geography => Some(FieldProtoType::String as i32),
                TableFieldType::Numeric => Some(FieldProtoType::Bytes as i32),
                TableFieldType::Bignumeric => Some(FieldProtoType::String as i32),
                TableFieldType::Interval => Some(FieldProtoType::Sint64 as i32),
                TableFieldType::Json => Some(FieldProtoType::String as i32),
                TableFieldType::Range => todo!(),
                TableFieldType::Unspecified => None,
            },
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }
    }
}

impl fmt::Debug for FieldInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (field, mode, ty) = unpack(self.packed);

        f.debug_struct("Field")
            .field("name", &self.name)
            .field("mode", &mode)
            .field("ty", &ty)
            .field("field", &field)
            .finish()
    }
}

#[inline]
fn pack(field: Field, mode: Mode, ty: TableFieldType) -> u16 {
    let packed_mode_ty = (ty as i32 as u8) << 2 | (mode as i32 as u8);

    u16::from_be_bytes([field.to_byte(), packed_mode_ty])
}

#[inline]
fn unpack(packed: u16) -> (Field, Mode, TableFieldType) {
    let [field_byte, packed_mode_ty] = packed.to_be_bytes();

    let field = Field::from_byte(field_byte).expect("should be valid");

    let ty = TableFieldType::try_from((packed_mode_ty >> 2) as i32)
        .unwrap_or(TableFieldType::Unspecified);

    let mode = Mode::try_from((packed_mode_ty & 0b11) as i32).unwrap_or(Mode::Unspecified);

    (field, mode, ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::WireType;

    const WIRE_TYPES: &[WireType] = &[
        WireType::Bits32,
        WireType::Bits64,
        WireType::LengthDelimited,
        WireType::Varint,
    ];

    const MODES: &[Mode] = &[
        Mode::Unspecified,
        Mode::Nullable,
        Mode::Repeated,
        Mode::Required,
    ];

    const TYPES: &[TableFieldType] = &[
        TableFieldType::Unspecified,
        TableFieldType::Bool,
        TableFieldType::String,
        TableFieldType::Int64,
        TableFieldType::Double,
        TableFieldType::Struct,
        TableFieldType::Bytes,
        TableFieldType::Timestamp,
        TableFieldType::Date,
        TableFieldType::Time,
        TableFieldType::Datetime,
        TableFieldType::Geography,
        TableFieldType::Numeric,
        TableFieldType::Bignumeric,
        TableFieldType::Interval,
        TableFieldType::Json,
        TableFieldType::Range,
    ];

    #[test]
    fn test_pack_unpack() {
        for index in 0..(u8::MAX >> 3) {
            for wire_type in WIRE_TYPES.iter().cloned() {
                for mode in MODES.iter().copied() {
                    for ty in TYPES.iter().copied() {
                        let raw_mode = mode as i32 as u8;
                        assert_eq!(Mode::try_from(raw_mode as i32), Ok(mode));

                        let raw_ty = ty as i32 as u8;
                        assert_eq!(TableFieldType::try_from(raw_ty as i32), Ok(ty));

                        let field = Field::new(index as u8, wire_type);
                        let packed = pack(field, mode, ty);
                        let (unpacked_field, unpacked_mode, unpacked_ty) = unpack(packed);

                        assert_eq!(unpacked_field, field);
                        assert_eq!(unpacked_mode, mode);
                        assert_eq!(unpacked_ty, ty);
                    }
                }
            }
        }
    }
}
