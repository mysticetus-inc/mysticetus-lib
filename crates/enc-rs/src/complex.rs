#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Complex {
    real: f64,
    imag: f64,
}

impl Complex {
    #[inline]
    pub const fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    pub fn real(&self) -> f64 {
        self.real
    }

    pub fn imag(&self) -> f64 {
        self.imag
    }

    pub fn magnitude_squared(&self) -> f64 {
        self.real * self.real + self.imag * self.imag
    }

    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }
}

impl serde::Serialize for Complex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(2)?;

        tup.serialize_element(&self.real)?;
        tup.serialize_element(&self.imag)?;

        tup.end()
    }
}
