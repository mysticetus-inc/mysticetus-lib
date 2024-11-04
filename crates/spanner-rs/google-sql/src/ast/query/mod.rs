use super::common::Cast;
use super::common::expression::Expression;
use super::common::ident::Ident;

pub mod select;

pub struct QueryStatement<'src> {
    pub statement_hints: StatementHints<'src>,
    pub table_hints: TableHints<'src>,
    pub join_hints: JoinHints,
    pub query_expr: QueryExpr<'src>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatementHints<'src> {
    pub use_additional_parallelism: Option<bool>,
    pub optimizer_version: Option<&'src str>,
    pub optimizer_statistics_package: Option<&'src str>,
    pub allow_distributed_merge: Option<bool>,
    pub lock_scanned_ranges: Option<LockScannedRanges>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LockScannedRanges {
    Exclusive,
    #[default]
    Shared,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TableHints<'src> {
    pub force_index: Option<&'src str>,
    pub group_by_scan_optimization: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JoinHints {
    pub force_join_order: Option<bool>,
    pub join_method: Option<JoinMethod>,
    pub hash_join_build_side: Option<HashJoinBuildSide>,
    pub batch_mode: Option<bool>,
    pub hash_join_execution: Option<HashJoinExecution>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinMethod {
    HashJoin,
    ApplyJoin,
    MergeJoin,
    PushBroadcastHashJoin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashJoinBuildSide {
    BuildLeft,
    BuildRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashJoinExecution {
    MultiPass,
    OnePass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryExpr<'src> {
    pub cte: Vec<Cast<Ident<'src>, QueryExpr<'src>>>,
    pub kind: Box<QueryKind<'src>>,
    pub order_by: Vec<OrderBy<'src>>,
    pub limit: Option<Limit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy<'src> {
    expr: Expression<'src>,
    ascending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limit {
    count: usize,
    offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryKind<'src> {
    Select(select::Select<'src>),
    SubQuery(QueryExpr<'src>),
    SetOperation(SetOperation<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetOperation<'src> {
    lhs: QueryExpr<'src>,
    operator: SetOperator,
    rhs: QueryExpr<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetOperator {
    kind: SetOperatorKind,
    scope: Scope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOperatorKind {
    Union,
    Intersect,
    Except,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    All,
    Distinct,
}
