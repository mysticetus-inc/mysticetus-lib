use std::{fmt, num::FpCategory};

use serde::ser;

pub struct JsonSerializer<'a, W: ?Sized> {
    writer: &'a mut W,
    int_buf: itoa::Buffer,
    float_buf: ryu::Buffer,
}

impl<'a, W: fmt::Write + ?Sized> JsonSerializer<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            int_buf: itoa::Buffer::new(),
            float_buf: ryu::Buffer::new(),
        }
    }

    #[inline]
    fn serialize_int(&mut self, int: impl itoa::Integer) -> Result<(), Error> {
        self.writer
            .write_str(itoa::Buffer::new().format(int))
            .map_err(Error)
    }

    #[inline]
    fn serialize_float(&mut self, float: f64) -> Result<(), Error> {
        let s = match float.classify() {
            FpCategory::Infinite if float.is_sign_negative() => "\"-Inf\"",
            FpCategory::Infinite => "\"Inf\"",
            FpCategory::Nan => "\"NaN\"",
            FpCategory::Zero => "0",
            FpCategory::Normal | FpCategory::Subnormal => self.float_buf.format_finite(float),
        };

        self.writer.write_str(s).map_err(Error)
    }
}

#[derive(Debug)]
struct Error(fmt::Error);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        Self(fmt::Error::default())
    }
}

impl<'a, 'w, W: fmt::Write + ?Sized> serde::Serializer for &'a mut JsonSerializer<'w, W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = JsonArraySerializer<'a, 'w, W>;
    type SerializeTuple = JsonArraySerializer<'a, 'w, W>;
    type SerializeTupleStruct = JsonArraySerializer<'a, 'w, W>;

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_float(v as f64)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.serialize_float(v)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "\"{}\"", v.escape_debug()).map_err(Error)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0_u8; 4];
        self.writer.write_str("\"").map_err(Error)?;
        self.writer
            .write_str(v.encode_utf8(&mut buf))
            .map_err(Error)?;
        self.writer.write_str("\"").map_err(Error)
    }

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        match v {
            true => self.writer.write_str("true").map_err(Error),
            false => self.writer.write_str("false").map_err(Error),
        }
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_str("null").map_err(Error)
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.writer.write_char('[').map_err(Error)?;
        Ok(JsonArraySerializer {
            serializer: self,
            started: false,
        })
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        todo!()
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

pub struct JsonArraySerializer<'a, 'b, W: ?Sized> {
    serializer: &'a mut JsonSerializer<'b, W>,
    started: bool,
}

impl<W: fmt::Write + ?Sized> ser::SerializeSeq for JsonArraySerializer<'_, '_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        if self.started {
            self.serializer.writer.write_char(',').map_err(Error)?;
        }

        value.serialize(&mut *self.serializer)?;
        self.started = true;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.writer.write_char(']').map_err(Error)
    }
}

impl<W: fmt::Write + ?Sized> ser::SerializeTuple for JsonArraySerializer<'_, '_, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

impl<W: fmt::Write + ?Sized> ser::SerializeTupleStruct for JsonArraySerializer<'_, '_, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}
