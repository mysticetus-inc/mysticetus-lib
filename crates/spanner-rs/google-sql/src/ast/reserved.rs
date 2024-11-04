use super::{FromToken, UnexpectedToken};
use crate::Error;
// use crate::map::Mapping;
use crate::tokens::{Span, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DataType {
    Array,
    Bool,
    Bytes,
    Date,
    Float64,
    Int64,
    Json,
    Numeric,
    String,
    Struct,
    Timestamp,
}

peg::parser! {
    grammar types() for str {
        pub rule dtype() -> DataType
            = n:$(['a'..='z' | 'A'..='Z']) {? DataType::from_str(n).ok_or("invalid data type") }
    }
}

impl PartialOrd for DataType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DataType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = *self as u8;
        let b = *other as u8;
        a.cmp(&b)
    }
}

/*
static DATA_TYPE_MAPPING: Mapping<11, UniCase<&'static str>, DataType> = crate::map::mapping! {
    "ARRAY" => DataType::Array,
    "BOOL" => DataType::Bool,
    "BYTES" => DataType::Bytes,
    "DATE" => DataType::Date,
    "FLOAT64" => DataType::Float64,
    "INT64" => DataType::Int64,
    "JSON" => DataType::Json,
    "NUMERIC" => DataType::Numeric,
    "STRING" => DataType::String,
    "STRUCT" => DataType::Struct,
    "TIMESTAMP" => DataType::Timestamp,
};
*/

impl DataType {
    pub fn from_str(_s: &str) -> Option<Self> {
        // DATA_TYPE_MAPPING.get(s).copied()
        todo!()
    }

    fn as_str(&self) -> &'static str {
        /*
            DATA_TYPE_MAPPING
                .get_reverse(self)
                .expect("all variants of DataType are in the map")
                .as_ref()
        */
        todo!()
    }
}

impl<'src> FromToken<'src> for DataType {
    fn from_token(span: Span, token: Token<'src>) -> Result<Self, Error<'src>> {
        token
            .as_data_type()
            .ok_or_else(|| UnexpectedToken::new_expected(span, token, "a data type").into())
    }
}
impl<'src> FromToken<'src> for Keyword {
    fn from_token(span: Span, token: Token<'src>) -> Result<Self, Error<'src>> {
        token
            .as_keyword()
            .ok_or_else(|| UnexpectedToken::new_expected(span, token, "a keyword").into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Keyword {
    All,
    And,
    Any,
    Array,
    As,
    Asc,
    AssertRowsModified,
    At,
    Between,
    By,
    Case,
    Cast,
    Collate,
    Contains,
    Create,
    Cross,
    Cube,
    Current,
    Default,
    Define,
    Desc,
    Distinct,
    Else,
    End,
    Enum,
    Escape,
    Except,
    Exclude,
    Exists,
    Extract,
    False,
    Fetch,
    Following,
    For,
    From,
    Full,
    Group,
    Grouping,
    Groups,
    Hash,
    Having,
    If,
    Ignore,
    In,
    Inner,
    Intersect,
    Interval,
    Into,
    Is,
    Join,
    Lateral,
    Left,
    Like,
    Limit,
    Lookup,
    Merge,
    Natural,
    New,
    No,
    Not,
    Null,
    Nulls,
    Of,
    On,
    Or,
    Order,
    Outer,
    Over,
    Partition,
    Preceding,
    Proto,
    Range,
    Recursive,
    Respect,
    Right,
    Rollup,
    Rows,
    Select,
    Set,
    Some,
    Struct,
    Tablesample,
    Then,
    To,
    Treat,
    True,
    Unbounded,
    Union,
    Unnest,
    Using,
    When,
    Where,
    Window,
    With,
    Within,
}

impl Keyword {
    pub const fn const_cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = *self as u8;
        let b = *other as u8;

        if a > b {
            std::cmp::Ordering::Greater
        } else if a < b {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}
impl PartialOrd for Keyword {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Keyword {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.const_cmp(other)
    }
}

/*
static KW_MAPPING: Mapping<94, UniCase<&'static str>, Keyword> = crate::map::mapping! {
    "ALL" => Keyword::All,
    "AND" => Keyword::And,
    "ANY" => Keyword::Any,
    "ARRAY" => Keyword::Array,
    "AS" => Keyword::As,
    "ASC" => Keyword::Asc,
    "ASSERT_ROWS_MODIFIED" => Keyword::AssertRowsModified,
    "AT" => Keyword::At,
    "BETWEEN" => Keyword::Between,
    "BY" => Keyword::By,
    "CASE" => Keyword::Case,
    "CAST" => Keyword::Cast,
    "COLLATE" => Keyword::Collate,
    "CONTAINS" => Keyword::Contains,
    "CREATE" => Keyword::Create,
    "CROSS" => Keyword::Cross,
    "CUBE" => Keyword::Cube,
    "CURRENT" => Keyword::Current,
    "DEFAULT" => Keyword::Default,
    "DEFINE" => Keyword::Define,
    "DESC" => Keyword::Desc,
    "DISTINCT" => Keyword::Distinct,
    "ELSE" => Keyword::Else,
    "END" => Keyword::End,
    "ENUM" => Keyword::Enum,
    "ESCAPE" => Keyword::Escape,
    "EXCEPT" => Keyword::Except,
    "EXCLUDE" => Keyword::Exclude,
    "EXISTS" => Keyword::Exists,
    "EXTRACT" => Keyword::Extract,
    "FALSE" => Keyword::False,
    "FETCH" => Keyword::Fetch,
    "FOLLOWING" => Keyword::Following,
    "FOR" => Keyword::For,
    "FROM" => Keyword::From,
    "GROUP" => Keyword::Group,
    "GROUPING" => Keyword::Grouping,
    "GROUPS" => Keyword::Groups,
    "HASH" => Keyword::Hash,
    "HAVING" => Keyword::Having,
    "IF" => Keyword::If,
    "IGNORE" => Keyword::Ignore,
    "IN" => Keyword::In,
    "INNER" => Keyword::Inner,
    "INTERSECT" => Keyword::Intersect,
    "INTERVAL" => Keyword::Interval,
    "INTO" => Keyword::Into,
    "IS" => Keyword::Is,
    "JOIN" => Keyword::Join,
    "LATERAL" => Keyword::Lateral,
    "LEFT" => Keyword::Left,
    "LIKE" => Keyword::Like,
    "LIMIT" => Keyword::Limit,
    "LOOKUP" => Keyword::Lookup,
    "MERGE" => Keyword::Merge,
    "NATURAL" => Keyword::Natural,
    "NEW" => Keyword::New,
    "NO" => Keyword::No,
    "NOT" => Keyword::Not,
    "NULL" => Keyword::Null,
    "NULLS" => Keyword::Nulls,
    "OF" => Keyword::Of,
    "ON" => Keyword::On,
    "OR" => Keyword::Or,
    "ORDER" => Keyword::Order,
    "OUTER" => Keyword::Outer,
    "OVER" => Keyword::Over,
    "PARTITION" => Keyword::Partition,
    "PRECEDING" => Keyword::Preceding,
    "PROTO" => Keyword::Proto,
    "RANGE" => Keyword::Range,
    "RECURSIVE" => Keyword::Recursive,
    "RESPECT" => Keyword::Respect,
    "RIGHT" => Keyword::Right,
    "ROLLUP" => Keyword::Rollup,
    "ROWS" => Keyword::Rows,
    "SELECT" => Keyword::Select,
    "SET" => Keyword::Set,
    "SOME" => Keyword::Some,
    "STRUCT" => Keyword::Struct,
    "TABLESAMPLE" => Keyword::Tablesample,
    "THEN" => Keyword::Then,
    "TO" => Keyword::To,
    "TREAT" => Keyword::Treat,
    "TRUE" => Keyword::True,
    "UNBOUNDED" => Keyword::Unbounded,
    "UNION" => Keyword::Union,
    "UNNEST" => Keyword::Unnest,
    "USING" => Keyword::Using,
    "WHEN" => Keyword::When,
    "WHERE" => Keyword::Where,
    "WINDOW" => Keyword::Window,
    "WITH" => Keyword::With,
    "WITHIN" => Keyword::Within,
};
*/
impl Keyword {
    pub fn as_str(&self) -> &'static str {
        todo!()
        //KW_MAPPING
        //   .get_reverse(self)
        //   .expect("KW_MAPPING contains all variants of Keyword")
        //   .as_ref()
    }

    pub(crate) fn from_str(_s: &str) -> Option<Self> {
        // KW_MAPPING.get(s).copied()
        todo!()
    }
}
