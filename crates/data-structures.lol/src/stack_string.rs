use crate::stack::Stack;

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct StackString<const CAP: usize>(Stack<CAP, u8>);

impl<const CAP: usize> StackString<CAP> {
    #[inline]
    pub const fn new() -> Self {
        Self(Stack::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: this type enforces the inner Stack is valid UTF-8
        unsafe { std::str::from_utf8_unchecked(self.0.as_slice()) }
    }

    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        // SAFETY: this type enforces the inner Stack is valid UTF-8
        unsafe { std::str::from_utf8_unchecked_mut(self.0.as_mut_slice()) }
    }

    #[inline]
    pub fn push_str(&mut self, to_append: &str) -> Result<(), usize> {
        self.0.extend_from_slice_truncate(to_append.as_bytes())
    }

    pub fn push(&mut self, ch: char) -> Result<(), usize> {
        let mut buf = [0_u8; 4];
        ch.encode_utf8(&mut buf);

        let encoded = &buf[..ch.len_utf8()];

        if self.0.remaining_capacity() < encoded.len() {
            return Err(encoded.len() - self.0.remaining_capacity());
        }

        self.0.extend_from_slice(encoded);
        Ok(())
    }
}

impl<const CAP: usize> From<&str> for StackString<CAP> {
    fn from(value: &str) -> Self {
        let mut dst = Self::new();
        let _ = dst.push_str(value);
        dst
    }
}

impl<const CAP: usize> AsRef<str> for StackString<CAP> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAP: usize> AsMut<str> for StackString<CAP> {
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}
