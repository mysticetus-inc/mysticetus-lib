use super::{Deserializer, noop};
use crate::path::{ParsePathError, Path};

pub struct DeserializerBuilder<D> {
    pub(super) inner_de: D,
    pub(super) key_modifier: fn(&mut String),
    pub(super) root: Path,
}

impl Default for DeserializerBuilder<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl DeserializerBuilder<()> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            inner_de: (),
            key_modifier: noop,
            root: Path::Root,
        }
    }

    /// Insert the deserializer to be wrapped.
    pub fn with_deserializer<'de, D>(self, deserializer: D) -> DeserializerBuilder<D>
    where
        D: serde::Deserializer<'de>,
    {
        DeserializerBuilder {
            inner_de: deserializer,
            key_modifier: self.key_modifier,
            root: self.root,
        }
    }
}

impl<D> DeserializerBuilder<D> {
    /// Attach a function to modify keys as they get deserialized. Useful for unescaping keys
    /// with escaped characters.
    pub fn with_key_modifier(mut self, key_modifier: fn(&mut String)) -> Self {
        self.key_modifier = key_modifier;
        self
    }

    /// Insert a path that will prepend any paths generated from errors. Useful for nested data
    /// structures with lots of custom deserialization. Builds this root path with
    /// [`Path::from_str`], and this returns an error if that fails.
    pub fn starting_at<S>(mut self, path: S) -> Result<Self, ParsePathError>
    where
        S: AsRef<str>,
    {
        self.root = path.as_ref().parse::<Path>()?;
        Ok(self)
    }
}

impl<'de, D> DeserializerBuilder<D>
where
    D: serde::Deserializer<'de>,
{
    #[inline]
    pub fn build(self) -> Deserializer<D> {
        Deserializer::from(self)
    }
}
