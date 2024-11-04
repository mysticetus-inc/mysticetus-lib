use crate::ast::ParseTokens;
use crate::ast::common::AsAlias;
use crate::ast::common::expression::Expression;
use crate::ast::common::ident::Ident;
use crate::ast::query::select::Select;

pub struct InsertStatement<'src> {
    table: AsAlias<'src, Ident<'src>>,
    columns: Vec<Ident<'src>>,
    data: InsertData<'src>,
    return_clause: Vec<ReturnClause<'src>>,
}

pub enum InsertData<'src> {
    Values(Vec<Vec<ValueExpression<'src>>>),
    Select(Select<'src>),
}

pub enum ValueExpression<'src> {
    Default,
    Expression(Expression<'src>),
}

pub enum ReturnClause<'src> {
    SelectAll(SelectAll<'src>),
    Expr(AsAlias<'src, Expression<'src>>),
}

pub struct SelectAll<'src> {
    table_name: Option<Ident<'src>>,
    except: Vec<Ident<'src>>,
    replace: Vec<(Expression<'src>, Ident<'src>)>,
}

impl<'src> ParseTokens<'src> for ValueExpression<'src> {
    fn parse_tokens<T>(_tokens: &mut T) -> Result<Self, crate::Error<'src>>
    where
        T: crate::tokens::TokenizerKind<'src>,
    {
        todo!()
    }

    fn parse_optional<T>(_tokens: &mut T) -> Result<Option<Self>, crate::Error<'src>>
    where
        T: crate::tokens::TokenizerKind<'src>,
    {
        todo!()
    }
}
