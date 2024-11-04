use std::io::{self, BufRead, BufReader, Read};

use crate::Endianness;
use crate::iso8211::Iso8211Error;
use crate::slicing::ArraySlice;

const BUF_SIZE: usize = 128;

#[derive(Debug)]
pub struct Iso8211Reader<R> {
    inner: BufReader<R>,
    position: u64,
    // inline buffer for doing small read operations (aimed at multi-byte numbers, characters, etc)
    inline_buf: [u8; BUF_SIZE],
    // separate heap buffer for doing larger reads (aimed at read_until calls, strings, and longer
    // byte sequences)
    heap_buf: Vec<u8>,
}

macro_rules! impl_read_num {
    ($($fn_name:ident($out_ty:ty : $from_bytes_fn:ident)),* $(,)?) => {
        $(
            #[inline]
            pub fn $fn_name(&mut self) -> io::Result<$out_ty> {
                const N: usize = std::mem::size_of::<$out_ty>();

                let bytes = self.read_array::<N>()?;
                Ok(<$out_ty>::$from_bytes_fn(*bytes))
            }
        )*
    };
}

macro_rules! impl_read_num_endianness {
    ($($fn_name:ident($le_fn:ident, $be_fn:ident) -> $out_ty:ty),* $(,)?) => {
        $(
            #[inline]
            pub fn $fn_name(&mut self, endianness: Endianness) -> io::Result<$out_ty> {
                match endianness {
                    Endianness::Little => self.$le_fn(),
                    Endianness::Big => self.$be_fn(),
                }
            }
        )*
    };
}

impl<R: Read> Iso8211Reader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: BufReader::new(reader),
            position: 0,
            inline_buf: [0; BUF_SIZE],
            heap_buf: Vec::new(),
        }
    }

    pub fn peek_byte(&mut self) -> io::Result<Option<u8>> {
        let buf = self.fill_buf()?;
        Ok(buf.first().copied())
    }

    pub fn read_bytes_until(&mut self, until: u8) -> io::Result<&[u8]> {
        self.heap_buf.clear();
        let n_read = self.inner.read_until(until, &mut self.heap_buf)?;
        self.position += n_read as u64;

        // pop the terminator byte
        self.heap_buf.pop();

        Ok(self.heap_buf.as_slice())
    }

    pub fn read_bytes_until_either(
        &mut self,
        until_a: u8,
        until_b: u8,
    ) -> io::Result<(&[u8], Option<u8>)> {
        if until_a == until_b {
            // dont hold onto the response, we know it's just the heap_buf, that way we can call
            // has_data_left without borrowck errors.
            self.read_bytes_until(until_a)?;

            // since read_until ends as Ok at an EOF, we need to check for that case to return
            // the delimiter only if we didn't hit the EOF case.
            let delim_found = self.inner.has_data_left()?.then_some(until_a);
            Ok((self.heap_buf.as_slice(), delim_found))
        } else {
            self.heap_buf.clear();

            let (n_read, found) =
                read_until_either(&mut self.inner, until_a, until_b, &mut self.heap_buf)?;
            self.position += n_read as u64;

            // pop the trailing delim, unless we hit EOF
            if found.is_some() {
                self.heap_buf.pop();
            }

            Ok((self.heap_buf.as_slice(), found))
        }
    }

    pub fn read_str_until_either(
        &mut self,
        until_a: u8,
        until_b: u8,
    ) -> io::Result<(&str, Option<u8>)> {
        let (buf, found) = self.read_bytes_until_either(until_a, until_b)?;

        match std::str::from_utf8(buf) {
            Ok(b) => Ok((b, found)),
            Err(err) => {
                println!("{err}");
                println!("total len: {}", buf.len());
                println!("lossy: {}", String::from_utf8_lossy(buf));
                println!("invalid remainder: {:?}", &buf[err.valid_up_to()..]);
                Err(io::Error::new(io::ErrorKind::InvalidData, err))
            }
        }
    }

    pub fn read_str_until(&mut self, until: u8) -> io::Result<&str> {
        let bytes = self.read_bytes_until(until)?;

        match std::str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(err) => {
                println!("{err}");
                println!("total len: {}", bytes.len());
                println!("lossy: '{}'", String::from_utf8_lossy(bytes));
                println!("invalid remainder: {:?}", &bytes[err.valid_up_to()..]);
                Err(io::Error::new(io::ErrorKind::InvalidData, err))
            }
        }
    }

    #[inline]
    pub const fn position(&self) -> u64 {
        self.position
    }

    #[inline]
    pub(crate) fn parse_from_byte<T>(&mut self) -> Result<T, Iso8211Error>
    where
        T: crate::FromByte,
        T::Error: Into<Iso8211Error>,
    {
        let byte = self.read_byte()?;
        T::from_byte(byte).map_err(Into::into)
    }

    #[inline]
    pub(crate) fn parse_from_bytes<T>(&mut self, n_bytes: usize) -> Result<T, Iso8211Error>
    where
        T: crate::FromBytes,
        T::Error: Into<Iso8211Error>,
    {
        let bytes = self.read_bytes(n_bytes)?;
        T::from_bytes(bytes).map_err(Into::into)
    }

    #[inline]
    pub(crate) fn read_bytes(&mut self, n_bytes: usize) -> io::Result<&[u8]> {
        // try and use the inline buffer to avoid allocations.
        let dst = if n_bytes > self.inline_buf.len() {
            self.heap_buf.resize(n_bytes, 0);
            &mut self.heap_buf[..n_bytes]
        } else {
            &mut self.inline_buf[..n_bytes]
        };

        self.inner.read_exact(dst)?;
        self.position += n_bytes as u64;

        Ok(&*dst)
    }

    #[inline]
    pub(crate) fn read_sized_str(&mut self, n_bytes: usize) -> io::Result<&str> {
        let bytes = self.read_bytes(n_bytes)?;
        std::str::from_utf8(bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    #[inline]
    pub(crate) fn parse_from_array<T, const N: usize>(&mut self) -> Result<T, Iso8211Error>
    where
        T: crate::FromByteArray<N>,
        T::Error: Into<Iso8211Error>,
    {
        let buf = self.read_array::<N>()?;
        T::from_byte_array(buf).map_err(Into::into)
    }

    #[inline]
    pub fn read_array<const N: usize>(&mut self) -> io::Result<&[u8; N]> {
        let dst = self.inline_buf.leading_mut::<N>();

        self.inner.read_exact(dst)?;

        self.position += N as u64;

        Ok(dst)
    }

    impl_read_num! {
        read_u64_le(u64 : from_le_bytes),
        read_u64_be(u64 : from_be_bytes),
        read_u64_ne(u64 : from_ne_bytes),

        read_u32_le(u32 : from_le_bytes),
        read_u32_be(u32 : from_be_bytes),
        read_u32_ne(u32 : from_ne_bytes),

        read_u16_le(u16 : from_le_bytes),
        read_u16_be(u16 : from_be_bytes),
        read_u16_ne(u16 : from_ne_bytes),

        read_i64_le(i64 : from_le_bytes),
        read_i64_be(i64 : from_be_bytes),
        read_i64_ne(i64 : from_ne_bytes),

        read_i32_le(i32 : from_le_bytes),
        read_i32_be(i32 : from_be_bytes),
        read_i32_ne(i32 : from_ne_bytes),

        read_i16_le(i16 : from_le_bytes),
        read_i16_be(i16 : from_be_bytes),
        read_i16_ne(i16 : from_ne_bytes),

        read_f64_le(f64 : from_le_bytes),
        read_f64_be(f64 : from_be_bytes),
        read_f64_ne(f64 : from_ne_bytes),

        read_f32_le(f32 : from_le_bytes),
        read_f32_be(f32 : from_be_bytes),
        read_f32_ne(f32 : from_ne_bytes),
    }

    impl_read_num_endianness! {
        read_i8(read_i8_le, read_i8_be) -> i8,
        read_i16(read_i16_le, read_i16_be) -> i16,
        read_i32(read_i32_le, read_i32_be) -> i32,
        read_i64(read_i64_le, read_i64_be) -> i64,
        read_u16(read_u16_le, read_u16_be) -> u16,
        read_u32(read_u32_le, read_u32_be) -> u32,
        read_u64(read_u64_le, read_u64_be) -> u64,
        read_f32(read_f32_le, read_f32_be) -> f32,
        read_f64(read_f64_le, read_f64_be) -> f64,
    }

    pub fn read_i8_le(&mut self) -> io::Result<i8> {
        let b = self.read_byte()?;
        Ok(i8::from_le_bytes([b]))
    }

    pub fn read_i8_be(&mut self) -> io::Result<i8> {
        let b = self.read_byte()?;
        Ok(i8::from_be_bytes([b]))
    }

    #[inline]
    pub fn read_byte(&mut self) -> io::Result<u8> {
        let buf = self.inner.fill_buf()?;
        if let Some(byte) = buf.first().copied() {
            self.inner.consume(1);
            self.position += 1;
            Ok(byte)
        } else {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "eof while reading byte",
            ))
        }
    }

    pub fn read_char_digit(&mut self) -> Result<u8, Iso8211Error> {
        let b = self.read_byte()?;
        crate::utils::ascii_byte_to_digit(b).map_err(Into::into)
    }
}

impl<R: Read> BufRead for Iso8211Reader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
        self.position += amt as u64;
    }
}

impl<R: Read> Read for Iso8211Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(buf)?;
        self.position += read as u64;
        Ok(read)
    }
}

/// adapted from the std impl of [`BufRead::read_until`] to handle finding the first of either
/// delimiter
fn read_until_either<R: BufRead + ?Sized>(
    r: &mut R,
    delim_a: u8,
    delim_b: u8,
    buf: &mut Vec<u8>,
) -> io::Result<(usize, Option<u8>)> {
    let mut read = 0;
    loop {
        let (done, used) = {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };

            // essentially the only difference with the stdlib version, using memchr2 instead
            // memchr. instead of using a boolean flag to determine if we found it, we
            // use an option containing the byte that we found (if we found it).
            match memchr::memchr2(delim_a, delim_b, available) {
                Some(i) => {
                    buf.extend_from_slice(&available[..=i]);
                    (Some(available[i]), i + 1)
                }
                None => {
                    buf.extend_from_slice(available);
                    (None, available.len())
                }
            }
        };
        r.consume(used);
        read += used;
        if done.is_some() || used == 0 {
            return Ok((read, done));
        }
    }
}
