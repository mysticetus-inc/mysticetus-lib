/// basic protobuf encoding/decoding types/functions.
use std::fmt;

use bigquery_resources_rs::table::FieldType;
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Field {
    packed: u8,
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("wire_type", &self.wire_type())
            .field("field_number", &self.field_number())
            .finish()
    }
}

pub const fn field_type_to_wire_type(ft: FieldType) -> WireType {
    match ft {
        FieldType::String => WireType::LengthDelimited,
        FieldType::Bytes => WireType::LengthDelimited,
        FieldType::Integer => WireType::Varint,
        FieldType::Float => WireType::Bits64,
        FieldType::Bool => WireType::Varint,
        FieldType::Timestamp => WireType::Bits64,
        FieldType::Date => WireType::Varint,
        FieldType::Time => WireType::LengthDelimited,
        FieldType::DateTime => WireType::LengthDelimited,
        FieldType::Geography => WireType::LengthDelimited,
        FieldType::Numeric => WireType::LengthDelimited,
        FieldType::BigNumeric => WireType::LengthDelimited,
        FieldType::Json => WireType::LengthDelimited,
        FieldType::Record => WireType::LengthDelimited,
        FieldType::Range => WireType::LengthDelimited,
        FieldType::Interval => WireType::LengthDelimited,
    }
}

impl Field {
    pub const fn wire_type(&self) -> WireType {
        match WireType::from_byte(self.packed) {
            Ok(wt) => wt,
            Err(_) => {
                panic!("Field.packed should always be valid (checked when Field is constructed)")
            }
        }
    }

    pub const fn field_number(&self) -> u8 {
        self.packed >> 3
    }

    pub fn from_schema_field(field: &crate::write::FieldInfo) -> Result<Self, super::EncodeError> {
        let wire_type = field.wire_type()?;
        Ok(Self::new(field.index(), wire_type))
    }

    #[inline]
    pub const fn new(field_number: u8, wire_type: WireType) -> Self {
        let packed = (field_number << 3) | (wire_type as u8);
        Self { packed }
    }

    #[inline]
    pub const fn from_byte(byte: u8) -> Result<Self, DecodeError> {
        match WireType::from_byte(byte) {
            Ok(_) => Ok(Self { packed: byte }),
            Err(err) => Err(err),
        }
    }

    pub fn parse_from_buf<B>(buf: &mut B) -> Result<(Self, RawProtoValue), DecodeError>
    where
        B: Buf,
    {
        let byte = buf.get_u8();
        match WireType::from_byte_and_parse(byte, buf) {
            Ok((_, value)) => Ok((Self { packed: byte }, value)),
            Err(err) => Err(err),
        }
    }

    pub const fn to_byte(&self) -> u8 {
        self.packed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldPair {
    field: Field,
    value: RawProtoValue,
}

impl fmt::Display for FieldPair {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} ({:?}): {:?}",
            self.field.field_number(),
            self.field.wire_type(),
            self.value,
        )
    }
}

impl FieldPair {
    #[allow(unused)]
    pub fn from_buf<B>(buf: &mut B) -> Result<Self, DecodeError>
    where
        B: Buf,
    {
        let (field, value) = Field::parse_from_buf(buf)?;
        Ok(Self { field, value })
    }

    #[cfg(test)]
    pub fn encode<B>(&self, dst: &mut B) -> usize
    where
        B: BufMut,
    {
        dst.put_u8(self.field.to_byte());

        // tag byte + encoded value len
        self.value.encode(dst) + 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Varint(usize);

impl Varint {
    const MSB: u8 = 1 << 7;
    const MSB_MASK: u8 = !Self::MSB;

    pub const fn from_bool(b: bool) -> Self {
        Self(b as usize)
    }

    pub const fn from_unsigned(uint: usize) -> Self {
        Self(uint)
    }

    pub const fn from_signed(int: isize) -> Self {
        Self(zigzag::encode(int))
    }

    pub const fn as_uint(&self) -> usize {
        self.0
    }

    pub const fn as_bool(&self) -> Option<bool> {
        match self.0 {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    pub const fn as_int(&self) -> isize {
        self.0 as isize - 1 << std::mem::size_of::<isize>()
    }

    pub const fn as_sint(&self) -> isize {
        zigzag::decode(self.0)
    }

    pub fn encode<B>(&self, dst: &mut B) -> usize
    where
        B: BufMut,
    {
        let mut value = self.0;

        let mut bytes_inserted = 0;

        while value != 0 {
            let mut byte = (value % 255) as u8 & Varint::MSB_MASK;
            value >>= 7;

            if value != 0 {
                byte |= Varint::MSB;
            }

            dst.put_u8(byte);
            bytes_inserted += 1;
        }

        bytes_inserted
    }

    pub fn decode<B>(bytes: &mut B) -> Result<Self, DecodeError>
    where
        B: Buf,
    {
        if !Buf::has_remaining(bytes) {
            return Err(DecodeError::VarintEof);
        }

        #[inline]
        const fn inner(bytes: &[u8]) -> Result<(Varint, usize), DecodeError> {
            let mut result = 0;
            let mut offset = 0;

            while offset < bytes.len() {
                let b = bytes[offset];

                result |= ((b & Varint::MSB_MASK) as usize) << (7 * offset);
                offset += 1;

                if (b & Varint::MSB) != Varint::MSB {
                    return Ok((Varint(result), offset));
                }

                if 7 * offset >= 64 {
                    return Err(DecodeError::VarintTooLong);
                }
            }

            Err(DecodeError::VarintEof)
        }

        match inner(Buf::chunk(bytes)) {
            Ok((varint, consumed)) => {
                Buf::advance(bytes, consumed);
                Ok(varint)
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawProtoValue {
    Varint(Varint),
    Bits64(u64),
    LengthDelimited(Bytes),
    #[deprecated = "deprecated in the proto spec, here for completeness"]
    StartGroup,
    #[deprecated = "deprecated in the proto spec, here for completeness"]
    EndGroup,
    Bits32(u32),
}

impl fmt::Display for RawProtoValue {
    fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}

impl RawProtoValue {
    pub fn as_double(&self) -> Option<f64> {
        match self {
            Self::Bits64(bits) => Some(f64::from_bits(*bits)),
            _ => None,
        }
    }

    pub fn into_packed_varints(self) -> Result<PackedVarints, Self> {
        match self {
            Self::LengthDelimited(bytes) => Ok(PackedVarints { bytes }),
            _ => Err(self),
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Bits32(bits) => Some(f32::from_bits(*bits)),
            _ => None,
        }
    }

    pub fn as_sfixed32(&self) -> Option<i32> {
        match self {
            Self::Bits32(_) => todo!(),
            _ => None,
        }
    }

    pub fn as_sfixed64(&self) -> Option<i32> {
        match self {
            Self::Bits64(_) => todo!(),
            _ => None,
        }
    }

    pub fn as_fixed32(&self) -> Option<u32> {
        match self {
            Self::Bits32(int) => Some(*int),
            _ => None,
        }
    }

    pub fn as_fixed64(&self) -> Option<u64> {
        match self {
            Self::Bits64(int) => Some(*int),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Bits64(bits) => Some(f64::from_bits(*bits)),
            Self::Bits32(bits) => Some(f32::from_bits(*bits) as f64),
            _ => None,
        }
    }

    pub fn encode<B>(&self, dst: &mut B) -> usize
    where
        B: BufMut,
    {
        match self {
            Self::Varint(varint) => varint.encode(dst),
            Self::Bits64(int) => {
                dst.put_u64_le(*int);
                std::mem::size_of::<u64>()
            }
            Self::LengthDelimited(delim) => {
                let len = Varint::from_unsigned(delim.len());
                let encoded_len = len.encode(dst);
                dst.put_slice(delim.as_ref());
                encoded_len + delim.len()
            }
            #[allow(deprecated)]
            Self::StartGroup => todo!("start groups not supported"),
            #[allow(deprecated)]
            Self::EndGroup => todo!("end groups not supported"),
            Self::Bits32(int) => {
                dst.put_u32_le(*int);
                std::mem::size_of::<u32>()
            }
        }
    }
}

pub fn parse_length_delimited<B>(buf: &mut B) -> Result<Bytes, DecodeError>
where
    B: Buf,
{
    let len = Varint::decode(buf)?;

    let mut bytes = BytesMut::with_capacity(len.0);

    if buf.remaining() < len.0 {
        return Err(DecodeError::LengthDelimitedEof {
            found: buf.remaining(),
            expected: len.0,
        });
    }

    bytes.extend_from_slice(&buf.chunk()[..len.0]);
    buf.advance(len.0);

    Ok(bytes.freeze())
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WireType {
    Varint = 0,
    Bits64 = 1,
    LengthDelimited = 2,
    StartGroup = 3,
    EndGroup = 4,
    Bits32 = 5,
}

impl WireType {
    const MASK: u8 = 0b111;

    pub const fn from_byte(byte: u8) -> Result<Self, DecodeError> {
        match byte & WireType::MASK {
            0 => Ok(Self::Varint),
            1 => Ok(Self::Bits64),
            2 => Ok(Self::LengthDelimited),
            3 => Ok(Self::StartGroup),
            4 => Ok(Self::EndGroup),
            5 => Ok(Self::Bits32),
            unknown => Err(DecodeError::UnknownWireType(unknown)),
        }
    }

    pub fn from_byte_and_parse<B>(
        byte: u8,
        buf: &mut B,
    ) -> Result<(Self, RawProtoValue), DecodeError>
    where
        B: Buf,
    {
        match byte & WireType::MASK {
            0 => {
                let parsed = Varint::decode(buf)?;
                Ok((Self::Varint, RawProtoValue::Varint(parsed)))
            }
            1 => Ok((Self::Bits64, RawProtoValue::Bits64(buf.get_u64_le()))),
            2 => {
                let value = parse_length_delimited(buf)?;
                Ok((Self::LengthDelimited, RawProtoValue::LengthDelimited(value)))
            }
            3 => {
                todo!("start group not supported");
                // Ok((Self::StartGroup, RawProtoValue::StartGroup))
            }
            4 => {
                todo!("end group not supported");
                // Ok((Self::EndGroup, RawProtoValue::EndGroup))
            }
            5 => Ok((Self::Bits32, RawProtoValue::Bits32(buf.get_u32_le()))),
            unknown => Err(DecodeError::UnknownWireType(unknown)),
        }
    }

    pub fn parse_field<B>(&self, buf: &mut B) -> Result<RawProtoValue, DecodeError>
    where
        B: Buf,
    {
        match self {
            Self::Varint => Varint::decode(buf).map(RawProtoValue::Varint),
            Self::Bits64 => Ok(RawProtoValue::Bits64(buf.get_u64_le())),
            Self::LengthDelimited => {
                parse_length_delimited(buf).map(RawProtoValue::LengthDelimited)
            }
            Self::StartGroup => todo!(),
            Self::EndGroup => todo!(),
            Self::Bits32 => Ok(RawProtoValue::Bits32(buf.get_u32_le())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum DecodeError {
    #[error("ran out of bytes before varint ended")]
    VarintEof,
    #[error("varint surpassed 64 bits of endocoded data")]
    VarintTooLong,
    #[error("encountered unknown wire type: {0}. (should be 0 - 5 inclusive)")]
    UnknownWireType(u8),
    #[error("length delimited value expected {expected} bytes, but ran out at {found} bytes.")]
    LengthDelimitedEof { found: usize, expected: usize },
}

#[derive(Clone, PartialEq, Eq)]
pub struct PackedVarints {
    bytes: Bytes,
}

impl Iterator for PackedVarints {
    type Item = Result<Varint, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        Some(Varint::decode(&mut self.bytes))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.bytes.is_empty() {
            (0, Some(0))
        } else {
            (1, Some(self.bytes.len()))
        }
    }
}

/// zigzag encoding/decoding for signed integers.
///
/// Code replicates the following, straight from the Google/protobuf source:
/// https://github.com/protocolbuffers/protobuf/blob/c03eb88a87a1aac231c79971d817c36d2a2c2261/src/google/protobuf/wire_format_lite.h#L836-L880
///
/// Since rust has [`isize`]/[`usize`], there isn't a need to write separate 32 and 64 bit
/// versions.
pub mod zigzag {
    pub const fn encode(val: isize) -> usize {
        (val.cast_unsigned() << 1) ^ ((val >> isize::BITS - 1).cast_unsigned())
    }

    pub const fn decode(val: usize) -> isize {
        ((val >> 1) ^ (!(val & 1)).wrapping_add(1)).cast_signed()
    }

    #[test]
    fn test_zigzag() {
        use rand::Rng;
        use rand::distr::StandardUniform;

        const TESTS: usize = 10000;

        let mut rng = rand::rng();

        for int in (&mut rng)
            .sample_iter::<i64, _>(StandardUniform)
            .take(TESTS)
            .map(|int| int as isize)
        {
            assert_eq!(int, decode(encode(int)));
        }

        for uint in (&mut rng)
            .sample_iter::<u64, _>(StandardUniform)
            .take(TESTS)
            .map(|uint| uint as usize)
        {
            assert_eq!(uint, encode(decode(uint)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test helper that tests decoding + encoding, then returns the decoded field pair.
    fn test_encode_decode(expected: FieldPair, encoded: &'static [u8]) -> FieldPair {
        //  ---- Decode -----
        let mut buf = Bytes::from_static(encoded);
        let parsed = FieldPair::from_buf(&mut buf).unwrap();

        assert_eq!(expected, parsed);
        assert!(buf.is_empty());

        //  ---- Encode -----
        let mut dst = BytesMut::with_capacity(encoded.len());
        let encoded_bytes = expected.encode(&mut dst);

        assert_eq!(encoded.len(), encoded_bytes);
        assert_eq!(dst.as_ref(), encoded);

        parsed
    }

    #[test]
    fn test_varint2() {
        let ts = timestamp::Timestamp::now();

        let micros = ts.as_micros();

        let varint = Varint::from_signed(micros as _);

        assert_eq!(micros, varint.as_sint() as i64);
    }

    #[test]
    fn test_raw_varint_decode() {
        const VARINT_ENCODED: &[u8] = &[0b10101100, 0b00000010];
        const VARINT_DECODED: Varint = Varint(300);

        let mut bytes = Bytes::from_static(VARINT_ENCODED);
        let result = Varint::decode(&mut bytes).unwrap();

        assert_eq!(result, VARINT_DECODED);
        assert!(bytes.is_empty());
    }

    mod basic {
        use super::*;

        #[test]
        fn test_varint() {
            const PAIR: FieldPair = FieldPair {
                field: Field::new(1, WireType::Varint),
                value: RawProtoValue::Varint(Varint(150)),
            };

            const ENCODED_PAIR: &[u8] = &[0x08, 0x96, 0x01];

            test_encode_decode(PAIR, ENCODED_PAIR);
        }

        #[test]
        fn test_length_delimited() {
            const PAIR: FieldPair = FieldPair {
                field: Field::new(2, WireType::LengthDelimited),
                value: RawProtoValue::LengthDelimited(Bytes::from_static(b"testing")),
            };

            const ENCODED_PAIR: &[u8] = &[
                // field tag bytes
                0x12, // value length varint
                0x07, // value bytes
                0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67,
            ];

            test_encode_decode(PAIR, ENCODED_PAIR);
        }

        #[test]
        fn test_packed_varints() {
            let pair = FieldPair {
                field: Field::new(4, WireType::LengthDelimited),
                value: RawProtoValue::LengthDelimited(Bytes::from_static(&ENCODED_PAIR[2..])),
            };

            const ENCODED_PAIR: &[u8] = &[
                // tag (field number 4, wire type 2)
                0x22, // payload size (6 bytes)
                0x06, // first element (varint 3)
                0x03, // second element (varint 270)
                0x8E, 0x02, // third element (varint 86942)
                0x9E, 0xA7, 0x05,
            ];

            const EXPECTED_VARINTS: &[Varint] = &[
                Varint::from_unsigned(3),
                Varint::from_unsigned(270),
                Varint::from_unsigned(86942),
            ];

            let parsed = test_encode_decode(pair, ENCODED_PAIR);

            let packed_varints = parsed
                .value
                .into_packed_varints()
                .unwrap()
                .collect::<Result<Vec<Varint>, DecodeError>>()
                .unwrap();

            assert_eq!(EXPECTED_VARINTS.len(), packed_varints.len());
            assert_eq!(packed_varints.as_slice(), EXPECTED_VARINTS);
        }
    }
}
