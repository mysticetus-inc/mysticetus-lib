use std::num::NonZeroUsize;

use data_structures::non_empty_vec::NonEmptyVec;

use super::expression::Expression;
use super::ident::Ident;

#[derive(Debug, Clone, PartialEq)]
pub enum Operation<'src> {
    Access(AccessOperation<'src>),
    Binary(BinaryOperation<'src>),
    Unary(UnaryOperation<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccessOperation<'src> {
    Field {
        lhs: Box<Expression<'src>>,
        path: NonEmptyVec<Ident<'src>>,
    },
    Array {
        lhs: Box<Expression<'src>>,
        index: Index,
    },
    Json {
        lhs: Box<Expression<'src>>,
        subscript: JsonSubscript<'src>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonSubscript<'src> {
    Index(usize),
    FieldName(&'src str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Index {
    Offset(usize),
    SafeOffset(usize),
    Ordinal(NonZeroUsize),
    SafeOrdinal(NonZeroUsize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOperation<'src> {
    operator: UnaryOperator,
    expr: Box<Expression<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperation<'src> {
    sides: Box<Sides<'src>>,
    operator: BinaryOperator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sides<'src> {
    lhs: Expression<'src>,
    rhs: Expression<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOperator {
    UnaryPlus,
    UnaryNeg,
    BitwiseNot,
    IsNull,
    IsNotNull,
    IsTrue,
    IsNotTrue,
    IsFalse,
    IsNotFalse,
    LogicalNot,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOperator {
    Mul,
    Div,
    Concat,
    Add,
    Sub,
    LeftShift,
    RightShift,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    Eq,
    Lt,
    Gt,
    Lte,
    Gte,
    NotEq,
    Like,
    NotLike,
    Between,
    NotBetween,
    In,
    NotIn,
}
