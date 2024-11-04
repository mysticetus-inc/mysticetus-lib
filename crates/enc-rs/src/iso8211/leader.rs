use std::convert::TryFrom;
use std::fmt;
use std::num::NonZeroU8;

use data_structures::inline_str::InlineStr;

use crate::slicing::ArraySlice;
use crate::utils::{InvalidDigitByte, ascii_byte_to_digit, byte_enum, chars_to_usize};
use crate::{FromByte, FromByteArray};

/// Errors that can be found while parsing Data Descriptive Leaders and Data Leaders.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum LeaderError {
    #[error("invalid interchange level: {} (expected ' ', '1', '2' or '3')", *level as char)]
    InvalidInterchangeLevel { level: u8 },
    #[error("invalid leader identifier: {} (expected 'L', 'D' or 'R')", *ident as char)]
    InvalidIdentifier { ident: u8 },
    #[error("invalid version found: {} (expected ' ' or '1')", *version as char)]
    InvalidVersion { version: u8 },
    #[error("invalid record length bytes: {bytes:?}")]
    InvalidRecordLength { bytes: [u8; 5] },
    #[error(transparent)]
    InvalidDigitByte(#[from] InvalidDigitByte),
    #[error(transparent)]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("invalid entry map field '{}', found value: {}", field.as_str(), *value as char)]
    InvalidEntryMap { field: EntryMapField, value: u8 },
}

pub struct OptionalInterchangeLevel {
    interchange_level: Option<InterchangeLevel>,
}

impl FromByte for OptionalInterchangeLevel {
    type Error = LeaderError;

    fn from_byte(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            b' ' => Ok(Self {
                interchange_level: None,
            }),
            _ => match InterchangeLevel::from_byte(byte) {
                Ok(level) => Ok(Self {
                    interchange_level: Some(level),
                }),
                Err(error) => Err(error),
            },
        }
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
    pub enum InterchangeLevel {
        One = b'1',
        Two = b'2',
        Three = b'3',
    }
    from_byte_error(level: u8, const) -> LeaderError {
        LeaderError::InvalidInterchangeLevel { level }
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
    pub enum DataLeaderIdentifer {
        D = b'D',
        R = b'R',
    }
    from_byte_error(ident: u8, const) -> LeaderError {
        LeaderError::InvalidIdentifier { ident }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum EitherLeader {
    DataDescriptive(DataDescriptiveLeader),
    Data(DataLeader),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct DataLeader {
    pub(crate) record_length: u64,
    pub(crate) leader_identifier: DataLeaderIdentifer,
    pub(crate) base_address_of_field_area: u64,
    pub(crate) entry_map: EntryMap,
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
    pub enum VersionNumber {
        One = b'1',
    }

    from_byte_error(version: u8, const) -> LeaderError {
        LeaderError::InvalidVersion { version }
    }
}

pub trait Leader {
    fn record_length(&self) -> u64;

    fn entry_map(&self) -> &EntryMap;

    fn base_address_of_field_area(&self) -> u64;

    fn dictionary_entry_len(&self) -> usize {
        self.base_address_of_field_area() as usize - 24 + 1
    }
}

impl Leader for EitherLeader {
    fn record_length(&self) -> u64 {
        match self {
            Self::Data(dl) => dl.record_length,
            Self::DataDescriptive(ddl) => ddl.record_length,
        }
    }
    fn entry_map(&self) -> &EntryMap {
        match self {
            Self::Data(dl) => &dl.entry_map,
            Self::DataDescriptive(ddl) => &ddl.entry_map,
        }
    }

    fn base_address_of_field_area(&self) -> u64 {
        match self {
            Self::Data(dl) => dl.base_address_of_field_area,
            Self::DataDescriptive(ddl) => ddl.base_address_of_field_area,
        }
    }
}

impl Leader for DataLeader {
    fn record_length(&self) -> u64 {
        self.record_length
    }
    fn entry_map(&self) -> &EntryMap {
        &self.entry_map
    }

    fn base_address_of_field_area(&self) -> u64 {
        self.base_address_of_field_area
    }
}

impl Leader for DataDescriptiveLeader {
    fn record_length(&self) -> u64 {
        self.record_length
    }
    fn entry_map(&self) -> &EntryMap {
        &self.entry_map
    }

    fn base_address_of_field_area(&self) -> u64 {
        self.base_address_of_field_area
    }
}

impl crate::FromByteArray<24> for EitherLeader {
    type Error = LeaderError;

    fn from_byte_array(array: &[u8; 24]) -> Result<Self, Self::Error> {
        match &array[6] {
            &b'L' => DataDescriptiveLeader::from_byte_array(array).map(Self::DataDescriptive),
            &b'D' => {
                DataLeader::from_byte_array_and_ident(array, DataLeaderIdentifer::D).map(Self::Data)
            }
            &b'R' => {
                DataLeader::from_byte_array_and_ident(array, DataLeaderIdentifer::R).map(Self::Data)
            }
            other => Err(LeaderError::InvalidIdentifier { ident: *other }),
        }
    }
}

fn parse_record_length(bytes: &[u8; 5]) -> Result<u64, LeaderError> {
    match bytes[4] {
        b'0'..=b'9' => match chars_to_usize(bytes.as_slice()) {
            Ok(int) => Ok(int as u64),
            Err(_) => Err(LeaderError::InvalidRecordLength { bytes: *bytes }),
        },
        b'b' => Ok(u32::from_le_bytes(*bytes.leading::<4>()) as u64),
        b'B' => Ok(u32::from_be_bytes(*bytes.leading::<4>()) as u64),
        _ => Err(LeaderError::InvalidRecordLength { bytes: *bytes }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct DataDescriptiveLeader {
    record_length: u64,
    interchange_level: InterchangeLevel,
    inline_code_ext_indicator: char,
    version_number: Option<VersionNumber>,
    application_indicator: Option<char>,
    field_control_length: u16,
    base_address_of_field_area: u64,
    extended_char_set_indicator: InlineStr<3>,
    entry_map: EntryMap,
}

impl DataDescriptiveLeader {
    pub fn field_control_length(&self) -> usize {
        self.field_control_length as usize
    }
}

impl FromByteArray<24> for DataDescriptiveLeader {
    type Error = LeaderError;

    fn from_byte_array(bytes: &[u8; 24]) -> Result<Self, LeaderError> {
        let record_len_bytes = bytes.leading::<5>();
        let record_length = parse_record_length(record_len_bytes)?;

        let interchange_level = InterchangeLevel::from_byte(bytes[5])?;

        debug_assert!(bytes[6] == b'L');

        let inline_code_ext_indicator = bytes[7] as char;

        let version_number = match bytes[8] {
            b' ' => None,
            other => Some(VersionNumber::from_byte(other)?),
        };

        let application_indicator = match bytes[9] {
            b' ' => None,
            other => Some(other as char),
        };

        let field_control_length = chars_to_usize(&bytes[10..12])? as u16;
        let base_address_of_field_area = chars_to_usize(&bytes[12..17])? as u64;

        let extended_char_set_indicator = InlineStr::try_from(bytes.slice::<17, 3>())?;

        let entry_map = EntryMap::from_byte_array(bytes.slice::<20, 4>())?;

        Ok(Self {
            record_length,
            interchange_level,
            inline_code_ext_indicator,
            version_number,
            application_indicator,
            field_control_length,
            base_address_of_field_area,
            extended_char_set_indicator,
            entry_map,
        })
    }
}

impl FromByteArray<24> for DataLeader {
    type Error = LeaderError;
    fn from_byte_array(array: &[u8; 24]) -> Result<Self, Self::Error> {
        let ident = DataLeaderIdentifer::from_byte(array[6])?;
        Self::from_byte_array_and_ident(array, ident)
    }
}

impl DataLeader {
    fn from_byte_array_and_ident(
        bytes: &[u8; 24],
        leader_identifier: DataLeaderIdentifer,
    ) -> Result<Self, LeaderError> {
        let record_len_bytes = bytes.leading::<5>();
        let record_length = parse_record_length(record_len_bytes)?;

        // make sure we're actually this type of leader
        debug_assert!(matches!(bytes[6], b'R' | b'D'));

        // debug checks to make sure the DDR specific fields are indeed spaces
        // interchange level
        debug_assert_eq!(bytes[5], b' ');
        // inline code ext identifier
        debug_assert_eq!(bytes[7], b' ');
        // version number
        debug_assert_eq!(bytes[8], b' ');
        // application indicator
        debug_assert_eq!(bytes[9], b' ');
        // field control length
        debug_assert_eq!(&bytes[10..12], &[b' ', b' ']);
        // extended character set indicator
        debug_assert_eq!(&bytes[17..20], &[b' ', b' ', b' ']);

        let base_address_of_field_area = chars_to_usize(&bytes[12..17])? as u64;

        let entry_map = EntryMap::from_byte_array(bytes.slice::<20, 4>())?;

        Ok(Self {
            record_length,
            leader_identifier,
            base_address_of_field_area,
            entry_map,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryMapField {
    SizeOfLengthField,
    SizeOfPosField,
    SizeOfTagField,
}

impl EntryMapField {
    pub(crate) fn bounds(&self) -> std::ops::RangeInclusive<u8> {
        match self {
            Self::SizeOfLengthField => 1_u8..=9_u8,
            Self::SizeOfPosField => 1_u8..=9_u8,
            Self::SizeOfTagField => 1_u8..=7_u8,
        }
    }

    pub(crate) fn parse(&self, int: u8) -> Result<NonZeroU8, LeaderError> {
        if self.bounds().contains(&int) {
            // SAFETY: `bounds` only returns ranges between 1..=9 and 1..=7,
            // therefore any contained value is always non-zero.
            Ok(unsafe { NonZeroU8::new_unchecked(int) })
        } else {
            Err(LeaderError::InvalidEntryMap {
                field: *self,
                value: int,
            })
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::SizeOfLengthField => "size_of_length_field",
            Self::SizeOfPosField => "size_of_pos_field",
            Self::SizeOfTagField => "size_of_tag_field",
        }
    }
}

impl fmt::Display for EntryMapField {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct EntryMap {
    size_of_length_field: NonZeroU8,
    size_of_pos_field: NonZeroU8,
    size_of_tag_field: NonZeroU8,
}

impl EntryMap {
    pub fn size_of_length_field(&self) -> u8 {
        self.size_of_length_field.get()
    }

    pub fn size_of_pos_field(&self) -> u8 {
        self.size_of_pos_field.get()
    }

    pub fn size_of_tag_field(&self) -> u8 {
        self.size_of_tag_field.get()
    }

    pub fn total_size(&self) -> usize {
        self.size_of_length_field.get() as usize
            + self.size_of_pos_field.get() as usize
            + self.size_of_tag_field.get() as usize
    }
}

impl FromByteArray<4> for EntryMap {
    type Error = LeaderError;

    fn from_byte_array(bytes: &[u8; 4]) -> Result<Self, LeaderError> {
        let [len, pos, reserved, tag] = *bytes;

        debug_assert!(reserved == b'0');

        let length_int = ascii_byte_to_digit(len)?;
        let size_of_length_field = EntryMapField::SizeOfLengthField.parse(length_int)?;

        let pos_int = ascii_byte_to_digit(pos)?;
        let size_of_pos_field = EntryMapField::SizeOfPosField.parse(pos_int)?;

        let tag_int = ascii_byte_to_digit(tag)?;
        let size_of_tag_field = EntryMapField::SizeOfTagField.parse(tag_int)?;

        Ok(Self {
            size_of_length_field,
            size_of_pos_field,
            size_of_tag_field,
        })
    }
}
