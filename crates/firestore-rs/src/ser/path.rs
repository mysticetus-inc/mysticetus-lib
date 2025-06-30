use std::borrow::Cow;
use std::ops::Range;

use serde::ser::Error;

use crate::error::SerError;

#[derive(Default, Clone)]
pub(super) struct Path {
    buf: String,
    segments: Vec<Segment>,
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_path(f)
    }
}

impl Path {
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn push_borrowed(&mut self, key: &str) {
        let start = self.buf.len();
        self.buf.push_str(key);
        self.segments.push(Segment::Buf(start..self.buf.len()));
    }

    pub fn push_static(&mut self, key: &'static str) {
        self.segments.push(Segment::Static(key));
    }

    pub fn pop(&mut self) {
        if let Some(Segment::Buf(range)) = self.segments.pop() {
            assert_eq!(self.buf.len(), range.end);
            self.buf.truncate(range.start);
        }
    }

    pub fn pop_take(&mut self) -> Option<Cow<'static, str>> {
        let segment = self.segments.pop()?;

        match segment {
            Segment::Buf(range) => {
                assert_eq!(self.buf.len(), range.end);
                let owned = self.buf[range.start..range.end].to_owned();
                self.buf.truncate(range.start);
                Some(Cow::Owned(owned))
            }
            Segment::Static(stat) => Some(Cow::Borrowed(stat)),
        }
    }

    fn estimated_len(&self) -> usize {
        let separators = self.segments.len().saturating_sub(1);
        let raw_key_len = self.segments.iter().map(Segment::len).sum::<usize>();

        // arbitrary guess that half of the keys need to be escaped
        let escaping_len = 3 * (self.segments.len() / 2);

        separators + raw_key_len + escaping_len
    }

    pub fn make_path(&self) -> String {
        let mut s = String::with_capacity(self.estimated_len());
        self.format_path(&mut s)
            .expect("String fmt::Write impl should never fail");
        s
    }

    fn format_path<W: std::fmt::Write + ?Sized>(&self, dst: &mut W) -> std::fmt::Result {
        #[inline]
        fn format_segment<W: std::fmt::Write + ?Sized>(key: &str, dst: &mut W) -> std::fmt::Result {
            if super::component_needs_escaping(key) {
                todo!()
            } else {
                dst.write_str(key)
            }
        }

        // pull out the first segment, so we can append path
        // seprators ('.') unconditionally in the loop
        let mut segments = self.iter();

        let Some(first) = segments.next() else {
            return Ok(());
        };

        format_segment(first, dst)?;

        for segment in segments {
            dst.write_str(".")?;
            format_segment(segment, dst)?;
        }

        Ok(())
    }

    pub fn iter(&self) -> PathIter<'_> {
        PathIter {
            segments: self.segments.iter(),
            buf: &self.buf,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum Segment {
    Buf(Range<usize>),
    Static(&'static str),
}

impl Segment {
    fn len(&self) -> usize {
        match self {
            Self::Buf(range) => range.end - range.start,
            Self::Static(key) => key.len(),
        }
    }

    fn get<'a>(&'a self, buf: &'a str) -> &'a str {
        match self {
            Self::Buf(range) => &buf[range.start..range.end],
            Self::Static(key) => key,
        }
    }
}

pub(super) struct PathIter<'a> {
    segments: std::slice::Iter<'a, Segment>,
    buf: &'a str,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.segments.next().map(|seg| seg.get(self.buf))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.segments.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for PathIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.segments.next_back().map(|seg| seg.get(self.buf))
    }
}

impl ExactSizeIterator for PathIter<'_> {}

macro_rules! impl_simple_invalid_serialize_fns {
    ($($name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            fn $name(self, _: $arg_ty) -> Result<(), SerError> {
                Err(SerError::Serialize(serde::ser::Error::custom(concat!(
                    "'",
                    stringify!($arg_ty),
                    "' can't be a map key",
                ))))
            }
        )*
    };
}

impl serde::Serializer for &mut Path {
    type Ok = ();
    type Error = SerError;

    type SerializeSeq = serde::ser::Impossible<(), SerError>;
    type SerializeTuple = serde::ser::Impossible<(), SerError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), SerError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), SerError>;
    type SerializeMap = serde::ser::Impossible<(), SerError>;
    type SerializeStruct = serde::ser::Impossible<(), SerError>;
    type SerializeStructVariant = serde::ser::Impossible<(), SerError>;

    impl_simple_invalid_serialize_fns! {
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_f32(f32),
        serialize_f64(f64),
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.push_borrowed(v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let s = std::str::from_utf8(v)
            .map_err(|err| SerError::custom(format!("map key isn't valid utf8: {err}")))?;

        self.serialize_str(s)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'none' can't be a map key",
        ))))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'unit' can't be a map key",
        ))))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.push_static(name);
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.push_static(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'seq' can't be a map key",
        ))))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'tuple' can't be a map key",
        ))))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'tuple struct' can't be a map key",
        ))))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'tuple variant' can't be a map key",
        ))))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'map' can't be a map key",
        ))))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'struct' can't be a map key",
        ))))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerError::Serialize(serde::ser::Error::custom(concat!(
            "'struct variant' can't be a map key",
        ))))
    }
}
