#[derive(Debug, Clone)]
pub struct ProjectId(shared::Shared<str>);

impl ProjectId {
    #[inline]
    pub const fn new_static(project_id: &'static str) -> Self {
        Self(shared::Shared::Static(project_id))
    }

    pub fn new_shared(project_id: &str) -> Self {
        Self(shared::Shared::Shared(std::sync::Arc::from(project_id)))
    }

    pub fn new_shared_owned(project_id: String) -> Self {
        Self(shared::Shared::Shared(std::sync::Arc::from(
            project_id.into_boxed_str(),
        )))
    }

    pub fn new_cow(project_id: std::borrow::Cow<'_, str>) -> Self {
        match project_id {
            std::borrow::Cow::Borrowed(s) => Self::new_shared(s),
            std::borrow::Cow::Owned(s) => Self::new_shared_owned(s),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl<S> From<S> for ProjectId
where
    shared::Shared<str>: From<S>,
{
    #[inline]
    fn from(value: S) -> Self {
        Self(From::from(value))
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl<S: AsRef<str>> PartialEq<S> for ProjectId {
    #[inline]
    fn eq(&self, other: &S) -> bool {
        str::eq(self, other.as_ref())
    }
}

impl Eq for ProjectId {}

impl<S: AsRef<str>> PartialOrd<S> for ProjectId {
    #[inline]
    fn partial_cmp(&self, other: &S) -> Option<std::cmp::Ordering> {
        str::partial_cmp(self, other.as_ref())
    }
}

impl Ord for ProjectId {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        str::cmp(self, other)
    }
}

impl std::borrow::Borrow<str> for ProjectId {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl std::ops::Deref for ProjectId {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ProjectId {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}
