#[macro_export]
macro_rules! make_visitor {
    ($name:ident; $value:ty; $($t:tt)*) => {};
    ($value:ty; $($t:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_visitor_fn {
    () => {};
}

#[rustfmt::skip]
macro_rules! __make_visitor_inner__ {
    (
        name: [$($name:tt)*],
        value: [$($value:tt)*],
        generics: [$($generics:tt)*],
        bounds: [$($bounds:tt)*],
        expecting: [$($expecting:tt)*],
        visitor_lifetime: $de:lifetime,
    ) => {

        pub struct $($name)*<$de, $($generics)*>(std::marker::PhantomData<(&'de (), $($generics)*)>);

        impl<$de, $($generics)*> std::default::Default for $($name)*<$de, $($generics)*> {
            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }

        impl<$de, $($generics)*> $($name)*<$de, $($generics)*> {
            #[inline]
            pub const fn new() -> Self {
                Self(std::marker::PhantomData)
            }
        }

        impl<$de, $($generics)*> serde::de::Visitor<$de> for $($name)*<$de, $($generics)*>
        where
            $($bounds)*
        {
            type Value = $($value)*;

            #[inline]
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $($expecting)*)
            }
        }
    };
}

__make_visitor_inner__! {
    name: [Lol],
    value: [(A, B, C)],
    generics: [A, B, C],
    bounds: [A: Default, B: Clone, C: Send],
    expecting: ["a thingy {}", "lol"],
    visitor_lifetime: 'de,
}
