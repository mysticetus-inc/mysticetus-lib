pub mod delete;
pub mod insert;
pub mod update;

pub enum DmlStatement<'src> {
    Insert(insert::InsertStatement<'src>),
    Delete(delete::DeleteStatement<'src>),
    Update(update::UpdateStatement<'src>),
}
