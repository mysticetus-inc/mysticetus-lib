#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
pub struct Reference(pub(crate) String);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
pub struct ReferenceRef<'a>(pub(super) &'a str);

impl Reference {
    pub fn as_ref(&self) -> ReferenceRef<'_> {
        ReferenceRef(&self.0)
    }
}
