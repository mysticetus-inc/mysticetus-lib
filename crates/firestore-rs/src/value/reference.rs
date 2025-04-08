use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[repr(transparent)]
pub struct Reference(str);

impl<'de> serde::Deserialize<'de> for Box<Reference> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Reference::new_string)
    }
}

impl<'de> serde::Deserialize<'de> for &'de Reference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <&'de str as serde::Deserialize<'de>>::deserialize(deserializer).map(Reference::new)
    }
}

impl PartialOrd for Reference {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Reference {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        crate::util::cmp_paths(self.as_str(), other.as_str())
    }
}

impl AsRef<str> for Reference {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Reference {
    pub fn new(s: &str) -> &Self {
        // SAFETY: We're repr(transparent)
        unsafe { std::mem::transmute::<&str, &Self>(s) }
    }

    pub fn id(&self) -> &str {
        let s = self.as_str();
        s.rsplit_once('/').map(|(_, id)| id).unwrap_or(s)
    }

    pub(crate) fn to_string(&self) -> String {
        self.as_str().to_owned()
    }

    pub fn new_string(s: String) -> Box<Self> {
        Self::new_owned(s.into_boxed_str())
    }

    pub fn new_owned(s: Box<str>) -> Box<Self> {
        // SAFETY: We're repr(transparent)
        unsafe { std::mem::transmute::<Box<str>, Box<Self>>(s) }
    }

    pub fn into_boxed_str(self: Box<Self>) -> Box<str> {
        // SAFETY: We're repr(transparent)
        unsafe { std::mem::transmute(self) }
    }

    pub fn into_string(self: Box<Self>) -> String {
        Self::into_boxed_str(self).into_string()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Clone for Box<Reference> {
    fn clone(&self) -> Self {
        Reference::to_owned(self)
    }
}

impl ToOwned for Reference {
    type Owned = Box<Reference>;

    fn to_owned(&self) -> Self::Owned {
        Self::new_owned(Box::from(&self.0))
    }
}
