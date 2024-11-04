use super::common::ident::Ident;
use crate::tokens::StringLiteral;

pub enum DdlStatement<'src> {
    CreateDatabase { database_id: Ident<'src> },
    AlterDatabase(AlterDatabase<'src>),
}

pub struct AlterDatabase<'src> {
    pub database_id: Ident<'src>,
    pub options: AlterDbOptions<'src>,
}

pub struct AlterDbOptions<'src> {
    pub default_leader: Option<NullOr<DefaultLeader>>,
    pub optimizer_version: Option<NullOr<OptimizerVersion<'src>>>,
    pub optimizer_statistics_package: Option<NullOr<StringLiteral<'src>>>,
    pub version_retention_period: Option<NullOr<StringLiteral<'src>>>,
}

pub enum NullOr<T> {
    Option(T),
    Null,
}

pub enum DefaultLeader {
    Region,
}

pub enum OptimizerVersion<'src> {
    Version(&'src str),
}

peg::parser! {
    grammar ddl_parser() for str {
        rule unescaped_database_id() -> &'input str
            = s:$(['a'..='z'] ['a'..='z' | '0'..='9' | '-' | '_']*<1, 29>)
            {?
                if s.ends_with(|ch: char| matches!(ch, '-' | '_')) {
                    Err("can't end with a '-' or '_'")
                } else {
                    Ok(s)
                }
            }

        rule escaped_database_id() -> &'input str
            = s:$(['`'] ['_']*<2, 29> ['`']) { s }

        pub rule database_id() -> &'input str = unescaped_database_id() / escaped_database_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_id_parser() {
        ddl_parser::database_id("ContainsCaps").expect_err("should fail");
        ddl_parser::database_id("endswith-").expect_err("should fail");
        ddl_parser::database_id("1startswithnumber").expect_err("should fail");
        ddl_parser::database_id("a-valid_id1234").expect("should be valid");
    }
}
