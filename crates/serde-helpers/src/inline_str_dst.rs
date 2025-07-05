//! [`InlineStrDst`], an inline string buffer mainly for capturing short key names.

use std::str::Utf8Error;

use crate::string_dst::StringDst;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InlineStrDst<const CAP: usize> {
    buf: [u8; CAP],
    overflow_flag: bool,
    len: usize,
}

impl<const CAP: usize> Default for InlineStrDst<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> InlineStrDst<CAP> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            buf: [0; CAP],
            overflow_flag: false,
            len: 0,
        }
    }

    #[inline(always)]
    pub const fn has_overflowed(&self) -> bool {
        self.overflow_flag
    }

    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        if self.len == 0 {
            return Ok("");
        }

        std::str::from_utf8(&self.buf[..self.len])
    }

    /// # Safety
    /// This defers to [`std::str::from_utf8_unchecked`], so the invariants for that
    /// must be upheld.
    pub unsafe fn as_str_unchecked(&self) -> &str {
        if self.len == 0 {
            return "";
        }

        // SAFETY: function is unsafe, caller must uphold this
        unsafe { std::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
}

impl<const CAP: usize> StringDst for InlineStrDst<CAP> {
    type Ref<'a>
        = Result<&'a str, Utf8Error>
    where
        Self: 'a;

    fn len(&self) -> usize {
        self.len
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn get_ref(&self) -> Self::Ref<'_> {
        self.as_str()
    }

    fn handle_str(&mut self, string: &str) {
        let rest = &mut self.buf[self.len..];

        let to_copy = rest.len().min(string.len());
        rest[..to_copy].copy_from_slice(&string.as_bytes()[..to_copy]);

        self.len += to_copy;
        self.overflow_flag = to_copy < string.len();
    }
}

#[cfg(test)]
mod tests {

    use super::InlineStrDst;
    use crate::string_dst::StringDst;

    #[test]
    fn test_inline_str_dst() {
        let mut dst = InlineStrDst::<9>::new();

        // push a string that wont cause an overflow, and verify it got written correctly
        dst.handle_str("hello");
        assert_eq!(dst.as_str(), Ok("hello"));
        assert!(!dst.has_overflowed());

        // push a string that __will__ cause an overflow, and verify it truncated where
        // we expect, and that the overflow flag was tripped.
        dst.handle_str(" world");
        assert_eq!(dst.as_str(), Ok("hello wor"));
        assert!(dst.has_overflowed());

        // ensure that pushing while already at an 'overflow' does nothing
        dst.handle_str("!");
        assert_eq!(dst.as_str(), Ok("hello wor"));
        assert!(dst.has_overflowed());

        dst.clear();
        // ensure that clear reset everything properly
        assert_eq!(dst.as_str(), Ok(""));
        assert!(!dst.has_overflowed());
    }
}
