#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct ColSums {
    sum: f64,
    sum_squared: f64,
}

impl ColSums {
    pub fn add_term(&mut self, x: f64) {
        self.sum += x;
        self.sum_squared += x.powi(2);
    }
}

pub(crate) fn count_float_digits(num: f64) -> usize {
    let mut digits = num.is_sign_negative() as usize;

    // add number of digits from the non-decimal spots
    digits += num.abs().trunc().log10().round() as usize;

    let mut fract = num.fract();
    if fract != 0.0 {
        digits += 1; // decimal character

        while fract != 0.0 {
            fract = (fract * 10.0).fract();
            digits += 1;
        }
    }

    digits
}

/// Formatting trait/helper for integer types. Provices methods to cast to a [`f64`] (which is
/// how all numbers in excel are represented), plus methods to estimate the number of characters
/// that'll be "written" when writing to a [`Cell`].
pub trait Integer: Copy + Ord {
    fn is_negative(self) -> bool;

    fn as_float(self) -> f64;

    fn abs_log10(self) -> Option<u32>;

    fn num_fmt_chars(self) -> usize {
        // if below zero, we need to add 1 for the negative sign.
        let neg = self.is_negative() as usize;
        // then use the log10 of the absolute value to get the magnitude (i.e number of digits)
        // in the number. If 0, abs_log10 returns None, so we can unwrap to 1 since 0 is 1 "digit"
        // in the formatting sense.
        let num_digits = self.abs_log10().unwrap_or(1) as usize;

        // sum the two for the number fo characters
        neg + num_digits
    }
}

impl<T> Integer for std::num::Wrapping<T>
where
    T: Integer,
{
    fn is_negative(self) -> bool {
        self.0.is_negative()
    }

    fn as_float(self) -> f64 {
        self.0.as_float()
    }

    fn abs_log10(self) -> Option<u32> {
        self.0.abs_log10()
    }

    fn num_fmt_chars(self) -> usize {
        self.0.num_fmt_chars()
    }
}

macro_rules! impl_integer {
    (
        int => $($int:ty),* ;
        uint => $($uint:ty),* ;
    ) => {
        $(
            impl Integer for $int {
                fn is_negative(self) -> bool {
                    self < 0
                }

                fn as_float(self) -> f64 {
                    self as f64
                }

                fn abs_log10(self) -> Option<u32> {
                    self.unsigned_abs().checked_ilog10()
                }
            }
        )*
        $(
            impl Integer for $uint {
                fn is_negative(self) -> bool {
                    false
                }

                fn as_float(self) -> f64 {
                    self as f64
                }

                fn abs_log10(self) -> Option<u32> {
                    self.checked_ilog10()
                }
            }
        )*
    };
}

impl_integer! {
    int => i8, i16, i32, i64, i128, isize;
    uint => u8, u16, u32, u64, u128, usize;
}

macro_rules! impl_nonzero_integers {
    ($($nonzero:ty),* $(,)?) => {
        $(
            impl Integer for $nonzero {
                fn is_negative(self) -> bool {
                    Integer::is_negative(self.get())
                }
                fn as_float(self) -> f64 {
                    self.get().as_float()
                }
                fn abs_log10(self) -> Option<u32> {
                    self.get().abs_log10()
                }
            }
        )*
    };
}

impl_nonzero_integers! {
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize,
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize,
}
