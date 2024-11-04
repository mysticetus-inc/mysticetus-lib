use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A number of degrees.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Degrees(f64);

impl Degrees {
    pub const ZERO: Self = Self(0.0);

    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[inline]
    pub const fn new(degrees: f64) -> Self {
        Self(degrees)
    }

    #[inline]
    pub fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        std::cmp::max_by(self, other, Self::total_cmp)
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        std::cmp::min_by(self, other, Self::total_cmp)
    }

    #[inline]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl std::fmt::Display for Degrees {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // unicode character is the degrees symbol
        write!(f, "{}\u{00B0}", self.0)
    }
}

impl Add for Degrees {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Degrees {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Sub for Degrees {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Degrees {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub(rhs);
    }
}

impl Div for Degrees {
    type Output = f64;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Neg for Degrees {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(self.0.neg())
    }
}

macro_rules! impl_scalar_mul_div {
    ($($rhs:ty),* $(,)?) => {
        $(
            impl Mul<$rhs> for Degrees {
                type Output = Self;

                #[inline]
                fn mul(self, rhs: $rhs) -> Self::Output {
                    Self(self.0 * rhs as f64)
                }
            }

            impl MulAssign<$rhs> for Degrees {
                #[inline]
                fn mul_assign(&mut self, rhs: $rhs) {
                    *self = self.mul(rhs);
                }
            }

            impl Div<$rhs> for Degrees {
                type Output = Self;

                #[inline]
                fn div(self, rhs: $rhs) -> Self::Output {
                    Self(self.0 / rhs as f64)
                }
            }

            impl DivAssign<$rhs> for Degrees {
                #[inline]
                fn div_assign(&mut self, rhs: $rhs) {
                    *self = self.div(rhs);
                }
            }
        )*
    };
}

impl_scalar_mul_div!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);
