//! Helper macros

macro_rules! checked_cast {
    ($item:expr; $item_type:ty[$item_var:ident] => $out_type:ty[$out_var:ident]) => {{
        if <$out_type>::MIN as $item_type <= $item && $item <= <$out_type>::MAX as $item_type {
            Ok($item as $out_type)
        } else {
            Err($crate::error::ConvertError::Overflow(
                $crate::error::OverflowType::Cast {
                    from: $crate::error::Num::$item_var,
                    to: $crate::error::Num::$out_var,
                },
            ))
        }
    }};
}

/// Helper for implementing math ops in [`std::ops`].
///
/// The syntax will look something like:
///
/// ```
/// // impl_math_ops! {
/// //     ImplementingType => {
/// //         TraitName => ((AssociatedType = AssociatedTypeValue))? => \
/// //              FnName(self: SelfTy, rhs: RhsType) (-> OutputType)?
/// //         {
/// //              self + rhs // or whatever logic is needed in the actual function block
/// //         },
/// //     }
/// // }
/// ```
///
/// A real example on real types would look like this (with the 'Self' shortcut syntax replaced
/// with the actual type for clarity).
///
/// ```compile_fail
/// // this doctest wont compile due to this macro being internal-only, which gives us no way to
/// // bring it into scope.
/// struct Seconds(i32);
/// struct Minutes(i32);
///
/// impl_math_ops! {
///     Seconds => {
///         Add => (Output = Seconds) => add(self: Seconds, rhs: Seconds) -> Seconds {
///             Seconds(self.0 + rhs.0)
///         },
///         AddAssign => add_assign(self: &mut Seconds, rhs: Seconds) {
///             *self = *self + rhs;
///         },
///         // Then, for a different type as the RHS:
///         Add => (Output = Seconds) => add(self: Seconds, rhs: Minutes) -> Seconds {
///             Seconds(self.0 + 60 * rhs.0)
///         },
///         AddAssign => add_assign(self: Seconds, rhs: Minutes) {
///             *self = *self + rhs;
///         },
///         // Then, an example of returning another type (i.e not 'Self')
///         Mul => (Output = Minutes) => mul(self: Seconds, rhs: Minutes) -> Minutes {
///             // not exactly a great way to multiply Unit of time together
///             Minutes((self.0 / 60) * rhs.0)
///         },
///         // ...
///     }
/// }
/// ```
///
/// See the usage on implementing math ops for [`Timestamp`] and [`Nanoseconds`] for more examples,
/// albeit as a wall of 'self's.
///
/// [`Timestamp`]: [`crate::Timestamp`]
/// [`Nanoseconds`]: [`crate::nanos::Nanoseconds`]
macro_rules! impl_math_ops {
    ($implementor:ident => {
        $(
            $op:ident => $(($assoc_ty:ident = $assoc_ty_val:ty) =>)?
            $fn_name:ident($self_id:ident: $self_ty:ty, $arg:ident: $arg_ty:ty)
            $(-> $out_ty:ty)? $fn_body:block
        ),* $(,)?
    }) => {
        $(
            impl ::std::ops::$op<$arg_ty> for $implementor {
                $( type $assoc_ty = $assoc_ty_val; )?

                fn $fn_name($self_id: $self_ty, $arg: $arg_ty) $(-> $out_ty)? {
                    $fn_body
                }
            }
        )*
    };
}

pub(crate) use {checked_cast, impl_math_ops};
