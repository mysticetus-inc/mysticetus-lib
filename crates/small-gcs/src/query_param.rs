/// Generic trait for different types of parameters.
pub trait Param: private::Sealed {}

impl<T: ?Sized> Param for T where T: private::Sealed {}

/// Specialized parameter trait for string-like parameters.
pub trait StringParam: Param<Target = str> {}

impl<T: ?Sized> StringParam for T where T: Param<Target = str> {}

#[allow(unused)]
/// Specialized parameter trait for integer-like parameters.
pub trait IntParam: Param<Target = i64> {}

impl<T: ?Sized> IntParam for T where T: Param<Target = i64> {}

mod private {
    use std::path::Path;

    pub trait Sealed {
        type Target: ?Sized;

        fn append_param(
            &self,
            param_name: &str,
            builder: reqwest::RequestBuilder,
        ) -> reqwest::RequestBuilder;
    }

    impl<T> Sealed for &T
    where
        T: Sealed + ?Sized,
    {
        type Target = T::Target;

        fn append_param(
            &self,
            param_name: &str,
            builder: reqwest::RequestBuilder,
        ) -> reqwest::RequestBuilder {
            T::append_param(self, param_name, builder)
        }
    }

    impl Sealed for bool {
        type Target = str;

        fn append_param(
            &self,
            param_name: &str,
            builder: reqwest::RequestBuilder,
        ) -> reqwest::RequestBuilder {
            (if *self { "true" } else { "false" }).append_param(param_name, builder)
        }
    }

    impl Sealed for Path {
        type Target = str;

        fn append_param(
            &self,
            param_name: &str,
            builder: reqwest::RequestBuilder,
        ) -> reqwest::RequestBuilder {
            builder.query(&[(param_name, self)])
        }
    }

    impl Sealed for char {
        type Target = str;

        fn append_param(
            &self,
            param_name: &str,
            builder: reqwest::RequestBuilder,
        ) -> reqwest::RequestBuilder {
            let mut buf = [0; 4];
            self.encode_utf8(&mut buf).append_param(param_name, builder)
        }
    }

    impl Sealed for str {
        type Target = str;

        fn append_param(
            &self,
            param_name: &str,
            builder: reqwest::RequestBuilder,
        ) -> reqwest::RequestBuilder {
            builder.query(&[(param_name, self)])
        }
    }

    impl Sealed for String {
        type Target = str;
        fn append_param(
            &self,
            param_name: &str,
            builder: reqwest::RequestBuilder,
        ) -> reqwest::RequestBuilder {
            str::append_param(self, param_name, builder)
        }
    }

    macro_rules! impl_for_ints {
        ($($t:ty),* $(,)?) => {
            $(
                impl Sealed for $t {
                    type Target = i64;

                    fn append_param(
                        &self,
                        param_name: &str,
                        builder: reqwest::RequestBuilder,
                    ) -> reqwest::RequestBuilder {
                        itoa::Buffer::new().format(*self).append_param(param_name, builder)
                    }
                }
            )*
        };
    }

    impl_for_ints! {
        u8, u16, u32, u64, u128, usize,
        i8, i16, i32, i64, i128, isize,
    }
}
