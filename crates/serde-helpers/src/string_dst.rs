use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::str::FromStr;

/// A trait to abstract over multiple ways to insert string references/owned strings.
pub trait StringDst {
    type Ref<'a>
    where
        Self: 'a;

    /// Get a reference to the inner type, which may or may not be a [`&str`].
    fn get_ref(&self) -> Self::Ref<'_>;

    /// Gets the length of the underlying dst, in bytes.
    fn len(&self) -> usize;

    /// Checks if the underlying dst is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Handle a [`&str`] reference.
    fn handle_str(&mut self, string: &str);

    /// Handle a [`&'static str`]. Default implementation forwards to [`handle_str`].
    /// This function exists to provide a more optimized implementation if the
    /// implementing type can take advantage of it.
    ///
    /// [`handle_str`]: StringDst::handle_str
    #[inline]
    fn handle_static_str(&mut self, string: &'static str) {
        self.handle_str(string);
    }

    /// Handle an owned [`String`]. The default impl calls [`StringDst::handle_str`], then drops
    /// the passed-in [`String`]. Types that can benefit from taking ownership of 'string' should
    /// override this.
    #[inline]
    fn handle_string(&mut self, string: String) {
        self.handle_str(&string);
    }

    /// Handle a single [`char`]. Default impl encodes the char to a [`str`], and
    /// defers to [`handle_str`].
    ///
    /// [`handle_str`]: StringDst::handle_str
    #[inline]
    fn handle_char(&mut self, ch: char) {
        let mut buf = [0; 4];
        self.handle_str(ch.encode_utf8(&mut buf));
    }

    /// Clear underlying buffer.
    fn clear(&mut self);
}

/// A [`StringDst`] implementing type that isn't an uderlying buffer, but instead
/// a [`FromStr`] parsable type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParsableDst<T: FromStr>(pub Option<Result<T, T::Err>>);

impl<T> StringDst for ParsableDst<T>
where
    T: FromStr,
{
    type Ref<'a>
        = &'a Option<Result<T, T::Err>>
    where
        T: 'a;

    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        &self.0
    }

    #[inline]
    fn len(&self) -> usize {
        match self.0 {
            Some(_) => 1,
            None => 0,
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.0 = None;
    }

    #[inline]
    fn handle_str(&mut self, string: &str) {
        self.0 = Some(string.parse::<T>());
    }

    #[inline]
    fn handle_string(&mut self, string: String) {
        self.handle_str(&string);
    }
}

// Main [`String`] impl

impl StringDst for String {
    type Ref<'a> = &'a str;

    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self
    }

    #[inline]
    fn len(&self) -> usize {
        // explicitely call the 'str' method to avoid calling the same func recursively
        str::len(self)
    }

    #[inline]
    fn handle_str(&mut self, string: &str) {
        self.push_str(string);
    }

    #[inline]
    fn handle_string(&mut self, string: String) {
        if self.is_empty() && self.capacity() < string.capacity() {
            *self = string;
        } else {
            self.handle_str(&string);
        }
    }

    #[inline]
    fn handle_char(&mut self, ch: char) {
        self.push(ch);
    }

    #[inline]
    fn clear(&mut self) {
        String::clear(self)
    }
}

// [`Cow<'_, str>`] impl
impl StringDst for Cow<'_, str> {
    type Ref<'a>
        = &'a str
    where
        Self: 'a;

    fn get_ref(&self) -> Self::Ref<'_> {
        self
    }
    #[inline]
    fn len(&self) -> usize {
        // explicitely call the 'str' method to avoid calling the same func recursively
        str::len(self)
    }

    #[inline]
    fn handle_str(&mut self, string: &str) {
        let trimmed = string.trim();

        if !trimmed.is_empty() {
            self.to_mut().handle_str(string);
        }
    }

    #[inline]
    fn handle_static_str(&mut self, string: &'static str) {
        if self.is_empty() {
            *self = Cow::Borrowed(string);
        } else {
            self.handle_str(string);
        }
    }

    #[inline]
    fn handle_string(&mut self, string: String) {
        match self {
            Cow::Borrowed("") => *self = Cow::Owned(string),
            Cow::Owned(s) => s.handle_string(string),
            _ => self.to_mut().handle_string(string),
        }
    }

    #[inline]
    fn clear(&mut self) {
        match self {
            Cow::Owned(s) => StringDst::clear(s),
            Cow::Borrowed(_) => *self = Cow::Borrowed(""),
        }
    }
}

// [`Option`] wrapped impl

impl<T> StringDst for Option<T>
where
    T: StringDst + Default + From<String>,
{
    type Ref<'a>
        = Option<T::Ref<'a>>
    where
        Self: 'a;

    #[inline]
    fn len(&self) -> usize {
        match self {
            Some(inner) => inner.len(),
            None => 0,
        }
    }

    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self.as_ref().map(|inner| inner.get_ref())
    }

    #[inline]
    fn handle_str(&mut self, string: &str) {
        let trimmed = string.trim();
        if !trimmed.is_empty() {
            self.get_or_insert_default().handle_str(trimmed);
        }
    }

    #[inline]
    fn handle_string(&mut self, string: String) {
        match self {
            Some(inner) => inner.handle_string(string),
            None => *self = Some(T::from(string)),
        }
    }

    #[inline]
    fn handle_static_str(&mut self, string: &'static str) {
        self.get_or_insert_default().handle_static_str(string)
    }

    #[inline]
    fn clear(&mut self) {
        if let Some(inner) = self.as_mut() {
            T::clear(inner);
        }
    }
}

impl<T> StringDst for RefCell<T>
where
    T: StringDst + ?Sized,
{
    type Ref<'a>
        = std::cell::Ref<'a, T>
    where
        T: 'a;

    fn len(&self) -> usize {
        self.borrow().len()
    }

    fn get_ref(&self) -> Self::Ref<'_> {
        self.borrow()
    }

    #[inline]
    fn handle_str(&mut self, string: &str) {
        self.get_mut().handle_str(string);
    }

    #[inline]
    fn handle_string(&mut self, string: String) {
        self.get_mut().handle_string(string);
    }

    #[inline]
    fn handle_char(&mut self, ch: char) {
        self.get_mut().handle_char(ch);
    }

    #[inline]
    fn handle_static_str(&mut self, string: &'static str) {
        self.get_mut().handle_static_str(string);
    }

    #[inline]
    fn clear(&mut self) {
        self.get_mut().clear();
    }
}

impl<T> StringDst for &RefCell<T>
where
    T: StringDst,
{
    type Ref<'a>
        = std::cell::Ref<'a, T>
    where
        Self: 'a;

    #[inline]
    fn len(&self) -> usize {
        self.borrow().len()
    }

    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self.borrow()
    }

    #[inline]
    fn handle_str(&mut self, string: &str) {
        self.borrow_mut().handle_str(string);
    }

    #[inline]
    fn handle_string(&mut self, string: String) {
        self.borrow_mut().handle_string(string);
    }

    #[inline]
    fn handle_char(&mut self, ch: char) {
        self.borrow_mut().handle_char(ch);
    }

    #[inline]
    fn handle_static_str(&mut self, string: &'static str) {
        self.borrow_mut().handle_static_str(string);
    }

    #[inline]
    fn clear(&mut self) {
        self.borrow_mut().clear();
    }
}

impl<T> StringDst for &Cell<T>
where
    T: StringDst + Default,
{
    type Ref<'a>
        = T
    where
        Self: 'a;

    #[inline]
    fn len(&self) -> usize {
        let s = self.take();
        let len = s.len();
        self.set(s);
        len
    }

    fn get_ref(&self) -> Self::Ref<'_> {
        self.take()
    }

    #[inline]
    fn handle_str(&mut self, string: &str) {
        let mut dst = self.take();
        dst.handle_str(string);
        self.set(dst);
    }

    #[inline]
    fn handle_string(&mut self, string: String) {
        let mut dst = self.take();
        dst.handle_string(string);
        self.set(dst);
    }

    #[inline]
    fn handle_char(&mut self, ch: char) {
        let mut dst = self.take();
        dst.handle_char(ch);
        self.set(dst);
    }

    #[inline]
    fn handle_static_str(&mut self, string: &'static str) {
        let mut dst = self.take();
        dst.handle_static_str(string);
        self.set(dst);
    }

    #[inline]
    fn clear(&mut self) {
        let mut dst = self.take();
        dst.clear();
        self.set(dst);
    }
}

// [`&mut T`] impl

impl<T> StringDst for &mut T
where
    T: StringDst,
{
    type Ref<'a>
        = T::Ref<'a>
    where
        Self: 'a;

    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        T::get_ref(self)
    }

    #[inline]
    fn len(&self) -> usize {
        T::len(self)
    }

    #[inline]
    fn handle_str(&mut self, string: &str) {
        T::handle_str(*self, string);
    }

    #[inline]
    fn handle_string(&mut self, string: String) {
        T::handle_string(*self, string);
    }

    #[inline]
    fn handle_static_str(&mut self, string: &'static str) {
        T::handle_static_str(*self, string);
    }

    #[inline]
    fn clear(&mut self) {
        T::clear(*self);
    }
}

macro_rules! assert_impl_string_dst {
    ($($t:ty),* $(,)?) => {
        const _: () = {
            const fn assert_impl<T>(_: std::marker::PhantomData<T>)
            where
                T: StringDst
            { }

            $(
                assert_impl::<$t>(std::marker::PhantomData);
            )*

        };
    };
}

assert_impl_string_dst! {
    String,
    &mut String,
    Cow<'_, str>,
    &mut Cow<'_, str>,
    Option<String>,
    Option<Cow<'static, str>>,
}
