macro_rules! read_into_string {
    ($src:expr; $stop_byte:expr) => {{
        let mut dst = Vec::with_capacity(10);

        for res in &mut $src {
            let byte = res?;
            if byte == $stop_byte {
                break;
            }

            dst.push(byte);
        }

        String::from_utf8(dst)?
    }};
}

/// Helper macro to define + implement FromByte for an enum with an implied #[repr(u8)].
macro_rules! byte_enum {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $($variant_name:ident = $byte:literal),*
            $(,)?
        }

        from_byte_error($bad_byte:ident: u8 $(, $const:tt)?) -> $err_ty:ty $build_error:block
    ) => {
        $(#[$attr])+
        #[repr(u8)]
        $vis enum $name {
            $($variant_name = $byte),*
        }

        impl $($const)? $crate::FromByte for $name {
            type Error = $err_ty;

            #[inline]
            fn from_byte(byte: u8) -> Result<Self, Self::Error> {
                match byte {
                    $($byte => Ok(Self::$variant_name),)*
                    $bad_byte => Err($build_error),
                }
            }
        }
    };
}

pub(crate) use byte_enum;

macro_rules! byte_iter_next {
    ($iter:expr; $eof:literal) => {{
        match $iter.next() {
            Some(Ok(byte)) => byte,
            Some(Err(err)) => return Err(err.into()),
            None => return Err(Iso8211Error::eof($eof)),
        }
    }};
    ($iter:expr) => {
        byte_iter_next!($iter; "unexpected EOF")
    };
}

pub const fn chars_to_usize(char_bytes: &[u8]) -> Result<usize, InvalidDigitByte> {
    if char_bytes.is_empty() {
        return Ok(0);
    }

    let mut digit_idx = char_bytes.len() as u32 - 1;

    let mut total = 0;

    let mut idx = 0;
    while idx < char_bytes.len() {
        let byte = char_bytes[idx];
        let digit = match ascii_byte_to_digit(byte) {
            Ok(b) => b as usize,
            Err(err) => return Err(err),
        };

        match digit_idx {
            0 => total += digit,
            1 => total += 10 * digit,
            2 => total += 100 * digit,
            3 => total += 1000 * digit,
            4 => total += 10000 * digit,
            5 => total += 100000 * digit,
            6 => total += 1000000 * digit,
            7 => total += 10000000 * digit,
            _ => total += 10usize.pow(digit_idx) * digit,
        };

        if digit_idx != 0 {
            digit_idx -= 1;
        }

        idx += 1;
    }

    Ok(total)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidDigitByte(pub(crate) u8);

impl std::error::Error for InvalidDigitByte {}

impl std::fmt::Display for InvalidDigitByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected an ascii digit, found '{}' ('{}_u8') instead",
            self.0 as char, self.0
        )
    }
}

impl From<u8> for InvalidDigitByte {
    fn from(ch: u8) -> Self {
        Self(ch)
    }
}

macro_rules! char_to_int {
    ($c:expr) => {{
        match $c {
            b'0' => 0,
            b'1' => 1,
            b'2' => 2,
            b'3' => 3,
            b'4' => 4,
            b'5' => 5,
            b'6' => 6,
            b'7' => 7,
            b'8' => 8,
            b'9' => 9,
            ch => return Err($crate::utils::InvalidDigitByte::from(ch).into()),
        }
    }};
}

macro_rules! read_byte_array {
    ($cnt:literal; $src:expr) => {{
        let mut buf = [0; $cnt];
        $src.read_exact(&mut buf)?;
        buf
    }};
}

/// Converts the the bytes representing ascii numeric characters into the actual numeric value.
///
/// ```
/// use enc_rs::utils::ascii_byte_to_digit;
///
/// assert_eq!(ascii_byte_to_digit(b'9'), Ok(9));
/// assert_eq!(ascii_byte_to_digit(b'2'), Ok(2));
/// assert!(ascii_byte_to_digit(b'A').is_err());
/// ```
#[inline]
pub const fn ascii_byte_to_digit(byte: u8) -> Result<u8, InvalidDigitByte> {
    if matches!(byte, b'0'..=b'9') {
        Ok(unsafe { ascii_byte_to_digit_unsafe(byte) })
    } else {
        Err(InvalidDigitByte(byte))
    }
}

pub const unsafe fn ascii_byte_to_digit_unsafe(byte: u8) -> u8 {
    byte - b'0'
}

#[cfg(test)]
mod test {
    super::byte_enum! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum TestEnum {
            A = b'a',
            B = b'b',
            C = b'c',
        }

        from_byte_error(bad_byte: u8) -> std::io::Error {
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, bad_byte.to_string())
        }
    }

    #[test]
    fn test_chars_to_usize() {
        let num = [b'1', b'2', b'3', b'9'];

        let zeros: [u8; 4] = [48, 48, 48, 48];

        assert_eq!(Ok(1239), super::chars_to_usize(&num));
        assert_eq!(Ok(0), super::chars_to_usize(&zeros));
    }

    #[test]
    fn test_byte_enum() {
        let a: TestEnum = crate::FromByte::from_byte(b'a').unwrap();
        assert_eq!(a, TestEnum::A);

        let b: TestEnum = crate::FromByte::from_byte(b'b').unwrap();
        assert_eq!(b, TestEnum::B);

        let c: TestEnum = crate::FromByte::from_byte(b'c').unwrap();
        assert_eq!(c, TestEnum::C);

        <TestEnum as crate::FromByte>::from_byte(b'd').expect_err("'d' not a variant");
        <TestEnum as crate::FromByte>::from_byte(b'A').expect_err("'A' not a variant");
    }
}
