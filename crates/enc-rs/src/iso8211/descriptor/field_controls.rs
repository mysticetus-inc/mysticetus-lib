use crate::FromByte;
use crate::slicing::ArraySlice;
use crate::utils::byte_enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct FieldControls {
    pub cardinality: Cardinality,
    pub data_type: DataType,
    pub aux_controls: [u8; 2],
    pub printable_graphics: [u8; 2],
    pub trunacted_escape_sequence: [u8; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum FieldControlError {
    #[error("invalid data type '{}' (expected and ascii digit, '0'..='6')", *byte as char)]
    InvalidDataType { byte: u8 },
    #[error("invalid field cardinality '{}' (expected and ascii digit, '0'..='3')", *byte as char)]
    InvalidCardinality { byte: u8 },
}

impl crate::FromByteArray<9> for FieldControls {
    type Error = FieldControlError;

    fn from_byte_array(array: &[u8; 9]) -> Result<Self, Self::Error> {
        let cardinality = Cardinality::from_byte(array[0])?;
        let data_type = DataType::from_byte(array[0])?;

        let aux_controls = *array.slice::<1, 2>();
        let printable_graphics = *array.slice::<3, 2>();
        let trunacted_escape_sequence = *array.slice::<5, 3>();

        Ok(Self {
            cardinality,
            data_type,
            aux_controls,
            printable_graphics,
            trunacted_escape_sequence,
        })
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
    pub enum Cardinality {
        Zero = b'0',
        One = b'1',
        TwoOrMore = b'2',
        Concatenated = b'3',
    }

    from_byte_error(byte: u8, const) -> FieldControlError {
        FieldControlError::InvalidCardinality { byte }
    }
}

impl Cardinality {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Zero => "  0  ",
            Self::One => "  1  ",
            Self::TwoOrMore => "  2+ ",
            Self::Concatenated => "concat",
        }
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
    pub enum DataType {
        CharacterString = b'0',
        ImplicitPoint = b'1',
        ExplicitPoint = b'2',
        ExplicitPointScaled = b'3',
        CharacterModeBitString = b'4',
        BitStringInclBinary = b'5',
        MixedDataTypes = b'6',
    }
    from_byte_error(byte: u8, const) -> FieldControlError {
        FieldControlError::InvalidDataType { byte }
    }
}
