use std::hash::Hash;
use std::num::NonZeroU8;

/// Wrapper newtype for a google project id.
///
/// Contains a (possibly leaked) static string.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ProjectId(&'static str);

impl ProjectId {
    #[inline]
    pub const fn new(project_id: &'static str) -> Self {
        #[cfg(debug_assertions)]
        assert!(matches!(validate_project_id(project_id.as_bytes()), Ok(())));

        Self(project_id)
    }

    pub const fn from_static_bytes(bytes: &'static [u8]) -> Result<Self, InvalidProjectId> {
        if !bytes.is_ascii() {
            return Err(InvalidProjectId::NotAscii);
        }

        // SAFETY: we checked above that this is valid ascii.
        let project_id = unsafe { std::str::from_utf8_unchecked(bytes) };

        Ok(Self(project_id))
    }

    pub fn from_byte_slice(bytes: &[u8]) -> Result<Self, InvalidProjectId> {
        if !bytes.is_ascii() {
            return Err(InvalidProjectId::NotAscii);
        }

        // SAFETY: we checked above that this is valid ascii.
        let project_id = unsafe { std::str::from_utf8_unchecked(bytes) };

        Ok(Self::from(Box::<str>::from(project_id)))
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

macro_rules! from_impls {
    (
        $(
            $str_ty:ty: |$s:ident| $make_static:expr
        ),*
        $(,)?
    ) => {
        $(
            impl From<$str_ty> for ProjectId {
                #[inline]
                fn from($s: $str_ty) -> Self {
                    Self($make_static)
                }
            }
        )*
    };
}

from_impls! {
    &'static str: |s| s,
    String: |s| Box::leak(s.into_boxed_str()),
    &String: |s| Box::leak(Box::from(s.as_str())),
    Box<str>: |s| Box::leak(s),
    std::sync::Arc<str>: |s| Box::leak(Box::from(s.as_ref())),
    std::borrow::Cow<'_, str>: |cow| match cow {
        std::borrow::Cow::Owned(s) => Box::leak(s.into_boxed_str()),
        std::borrow::Cow::Borrowed(b) => Box::leak(Box::from(b)),
    },
}

impl std::str::FromStr for ProjectId {
    type Err = InvalidProjectId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_project_id(s.as_bytes())?;
        Ok(Self::from(Box::<str>::from(s)))
    }
}

impl TryFrom<&ProjectId> for http::HeaderValue {
    type Error = http::header::InvalidHeaderValue;

    #[inline]
    fn try_from(value: &ProjectId) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<ProjectId> for http::HeaderValue {
    type Error = http::header::InvalidHeaderValue;

    #[inline]
    fn try_from(value: ProjectId) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl std::fmt::Display for ProjectId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl<S: AsRef<str>> PartialEq<S> for ProjectId {
    #[inline]
    fn eq(&self, other: &S) -> bool {
        str::eq(self.0, other.as_ref())
    }
}

impl Eq for ProjectId {}

impl<S: AsRef<str>> PartialOrd<S> for ProjectId {
    #[inline]
    fn partial_cmp(&self, other: &S) -> Option<std::cmp::Ordering> {
        str::partial_cmp(self.0, other.as_ref())
    }
}

impl Ord for ProjectId {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        str::cmp(self.0, other)
    }
}

impl Hash for ProjectId {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        str::hash(self.0, state)
    }
}

impl std::borrow::Borrow<str> for ProjectId {
    #[inline]
    fn borrow(&self) -> &str {
        self.0
    }
}

impl std::ops::Deref for ProjectId {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl AsRef<str> for ProjectId {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0
    }
}

const PROJECT_ID_MIN_LEN: usize = 6;
const PROJECT_ID_MAX_LEN: usize = 30;

/// Validates that a project id is valid, according to the rules that google as published here:
///
/// https://cloud.google.com/resource-manager/docs/creating-managing-projects#before_you_begin
///
/// The relevant rules that we check are:
///     - Must be 6 to 30 characters in length.
///     - Can only contain lowercase letters, numbers, and hyphens.
///     - Must start with a letter.
///     - Cannot end with a hyphen.
///     - Cannot contain restricted strings such as google and ssl (undefined and null also checked)
///
/// The rules say nothing about strings needing to be ascii only, but in order to make this const
/// that's what we'll assume. If in some future compiler version we can decode utf8 chars in
/// const time, we could relax this rule.
const fn validate_project_id(project_id: &[u8]) -> Result<(), InvalidProjectId> {
    const fn validate_len(len: usize) -> Result<(), InvalidProjectId> {
        // check lengths first
        match len {
            0 => return Err(InvalidProjectId::EmptyString),
            1..PROJECT_ID_MIN_LEN => {
                return Err(InvalidProjectId::TooShort {
                    len: NonZeroU8::new(len as u8).expect("too_short is between 1..6"),
                });
            }
            PROJECT_ID_MIN_LEN..PROJECT_ID_MAX_LEN => Ok(()),
            _ => {
                let len = if len > u8::MAX as usize {
                    None
                } else {
                    Some(NonZeroU8::new(len as u8).expect("this is between 31..=u8::MAX"))
                };

                return Err(InvalidProjectId::TooLong { len });
            }
        }
    }

    const fn check_for_invalid_substr(s: &[u8], offset: usize) -> Result<(), &'static str> {
        const fn substr_at_offset(bytes: &[u8], offset: usize, substr: &str) -> bool {
            if bytes.len() - offset < substr.len() {
                return false;
            }

            let mut substr_offset = 0;

            while substr_offset < substr.len() {
                if bytes[offset + substr_offset] != substr.as_bytes()[substr_offset] {
                    return false;
                }

                substr_offset += 1;
            }

            true
        }

        const PROJECT_ID_INVALID_SUBSTRINGS: &[&str] = &["google", "null", "ssl", "undefined"];

        let mut substr_idx = 0;

        while substr_idx < PROJECT_ID_INVALID_SUBSTRINGS.len() {
            let substr = PROJECT_ID_INVALID_SUBSTRINGS[substr_idx];

            if substr_at_offset(s, offset, substr) {
                return Err(substr);
            }

            substr_idx += 1;
        }

        Ok(())
    }

    if let Err(error) = validate_len(project_id.len()) {
        return Err(error);
    }

    if !project_id.is_ascii() {
        return Err(InvalidProjectId::NotAscii);
    }

    let first_char = project_id[0];
    if !matches!(first_char, b'a'..=b'z') {
        return Err(InvalidProjectId::FirstCharNotALetter(first_char));
    }

    if matches!(project_id.last(), Some(b'-')) {
        return Err(InvalidProjectId::LastCharHyphen);
    }

    // already validated the first char (since it has special rules)
    let mut i = 1;

    while i < project_id.len() {
        let ch = project_id[i];

        if !matches!(ch, b'a'..=b'z' | b'0'..=b'9' | b'-') {
            return Err(InvalidProjectId::InvalidChar { at: i as u8, ch });
        }

        if let Err(substr) = check_for_invalid_substr(project_id, i) {
            return Err(InvalidProjectId::ContainsInvalidSubstring {
                substr,
                at: i as u8,
            });
        }

        i += 1;
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InvalidProjectId {
    #[error("project id can't be empty ")]
    EmptyString,
    #[error("project id is too short ({len} characters), expected at least {PROJECT_ID_MIN_LEN}")]
    TooShort { len: NonZeroU8 },
    #[error("project id is too short ({} characters), expected {PROJECT_ID_MAX_LEN} or less", match .len {
        None => &">255" as &dyn std::fmt::Display,
        Some(len) => len as &dyn std::fmt::Display,
    })]
    TooLong {
        // in order to keep this enum compact, we use [None] to indicate
        // the (invalid) length was above 256, since we don't really need to
        // keep the exact size if something is invalid.
        len: Option<NonZeroU8>,
    },
    #[error("project id should be ascii")]
    NotAscii,
    #[error("first character isn't a lowercase letter: '{}'", *.0 as char)]
    FirstCharNotALetter(u8),
    #[error("last character can't be a hyphen")]
    LastCharHyphen,
    #[error("found invalid character at index {at}: '{}'", *.ch as char)]
    InvalidChar { at: u8, ch: u8 },
    #[error("contains invalid substring '{substr}' at index {at}")]
    ContainsInvalidSubstring { substr: &'static str, at: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_validations() {
        macro_rules! check_validation {
            ($s:literal) => {
                if let Err(error) = validate_project_id($s.as_bytes()) {
                    panic!("{:?} - {error:?}", $s);
                }
            };
            ($s:literal = $err_variant:ident $($rest_pattern:tt)*) => {{
                match validate_project_id($s.as_bytes()) {
                    Ok(()) => panic!(concat!(
                        "expected '",
                        $s,
                        "' to fail with an ",
                        stringify!($err_variant),
                        " error, instead it succeeded",
                    )),
                    Err(err) => {
                        if !matches!(err, InvalidProjectId::$err_variant $($rest_pattern)*) {
                            panic!("'{}' expected an {} error but got {err:?}", $s, stringify!($err_variant));
                        }
                    }
                }
            }};
        }

        check_validation!("mysticetus-oncloud");
        check_validation!("mysticetus-" = LastCharHyphen);
        check_validation!("" = EmptyString);
        check_validation!("_invalid" = FirstCharNotALetter(b'_'));
        check_validation!("utf8-project-id-ƒ" = NotAscii);
        check_validation!("short" = TooShort { len } if len.get() == 5);
        check_validation!(
            "way-too-long-way-too-long-way-too-long"
                = TooLong { len: Some(len) } if len.get() == 38
        );
        check_validation!(
            "invalid-substr-google" = ContainsInvalidSubstring {
                substr: "google",
                at: 15
            }
        );
    }
}
