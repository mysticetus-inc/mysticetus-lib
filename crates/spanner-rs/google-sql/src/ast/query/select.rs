use data_structures::non_empty_vec::NonEmptyVec;

use super::{QueryExpr, TableHints};
use crate::ast::common::expression::Expression;
use crate::ast::common::ident::{Ident, Path};
use crate::ast::common::{AsAlias, Cast};

#[derive(Debug, Clone, PartialEq)]
pub struct Select<'src> {
    scope: Option<super::Scope>,
    cast: Option<Cast<(), Target>>,
    select_list: Vec<SelectListItem<'src>>,
    from: NonEmptyVec<FromClause<'src>>,
    where_expr: Option<Expression<'src>>,
    group_by: Vec<Expression<'src>>,
    having: Option<Expression<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FromClause<'src> {
    item: FromItem<'src>,
    table_sample_operator: Option<TableSampleOperator>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FromItem<'src> {
    Table {
        name: Ident<'src>,
        hints: TableHints<'src>,
        alias: Option<Ident<'src>>,
    },
    Subquery(AsAlias<'src, QueryExpr<'src>>),
    Join(JoinOperation<'src>),
    FieldPath(Path<'src>),
    Unnest(UnnestOperator<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinOperation<'src> {
    _priv: &'src (),
}
#[derive(Debug, Clone, PartialEq)]
pub struct UnnestOperator<'src> {
    _priv: &'src (),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TableSampleOperator {
    method: TableSampleMethod,
    sample_size: SampleSize,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TableSampleMethod {
    Bernoulli,
    Reservoir,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SampleSize {
    Rows(usize),
    Percent(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Struct,
    Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectListItem<'src> {
    SelectAll(SelectAll<'src>),
    Expr(Cast<Expression<'src>, Option<Ident<'src>>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectAll<'src> {
    _lmao: &'src (),
}
