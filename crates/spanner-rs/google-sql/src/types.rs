pub(crate) struct Field {
    name: Option<String>,
    ty: DataType,
}

trait DataType2 {
    fn name(&self) -> &'static str;
}

pub(crate) enum DataType {
    Array(Box<DataType>),
    Bool,
    Bytes,
    Date,
    Float64,
    Int64,
    Json,
    Numeric,
    String,
    Struct(Vec<Field>),
    Timestamp,
}

impl DataType {
    pub(crate) const fn as_marker(&self) -> DataTypes {
        match self {
            Self::Array(_) => DataTypes::ARRAY,
            Self::Bool => DataTypes::BOOL,
            Self::Bytes => DataTypes::BYTES,
            Self::Date => DataTypes::DATE,
            Self::Float64 => DataTypes::FLOAT64,
            Self::Int64 => DataTypes::INT64,
            Self::Json => DataTypes::JSON,
            Self::Numeric => DataTypes::NUMERIC,
            Self::String => DataTypes::STRING,
            Self::Struct(_) => DataTypes::STRUCT,
            Self::Timestamp => DataTypes::TIMESTAMP,
        }
    }
}

bitflags::bitflags! {
    pub(crate) struct DataTypes: u16 {
        const ARRAY     = 0b00000000001;
        const BOOL      = 0b00000000010;
        const BYTES     = 0b00000000100;
        const DATE      = 0b00000001000;
        const FLOAT64   = 0b00000010000;
        const INT64     = 0b00000100000;
        const JSON      = 0b00001000000;
        const NUMERIC   = 0b00010000000;
        const STRING    = 0b00100000000;
        const STRUCT    = 0b01000000000;
        const TIMESTAMP = 0b10000000000;

        const ANY_NUMERIC = Self::INT64.bits | Self::FLOAT64.bits | Self::NUMERIC.bits;

        const NESTED = Self::ARRAY.bits | Self::STRUCT.bits | Self::JSON.bits;
    }
}

impl DataTypes {
    pub const COLUMN: Self = Self::all().difference(Self::NESTED);

    pub const GROUPABLE: Self = Self::COLUMN;
    pub const ORDERABLE: Self = Self::COLUMN;

    pub const COMPARABLE: Self = Self::all().difference(Self::JSON);
}
