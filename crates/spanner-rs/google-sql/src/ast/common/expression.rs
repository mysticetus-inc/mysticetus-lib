use super::operator::Operation;
use crate::Error;
use crate::ast::ParseTokens;
use crate::ast::query::QueryExpr;
use crate::tokens::TokenizerKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'src> {
    Value(ValueExpr<'src>),
    FunctionCall(FunctionCall<'src>),
    SubQuery(QueryExpr<'src>),
    Operation(Operation<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr<'src> {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'src str),
    Array(Vec<Expression<'src>>),
    StructTuple(Vec<Expression<'src>>),
}

impl<'src> ParseTokens<'src> for Expression<'src> {
    fn parse_tokens<T>(_tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall<'src> {
    pub safe: bool,
    pub name: &'src str,
    pub arguments: Option<Arguments<'src>>,
    pub function_hint: Option<FunctionHint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionHint {
    DisableInline(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arguments<'src> {
    arguments: Vec<Expression<'src>>,
    aggregate_options: Option<AggregateOptions<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AggregateOptions<'src> {
    distinct: bool,
    nulls: Option<NullHandling>,
    having: Option<Box<Having<'src>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullHandling {
    Ignore,
    Respect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Having<'src> {
    Min(Expression<'src>),
    Max(Expression<'src>),
}
