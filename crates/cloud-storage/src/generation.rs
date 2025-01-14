pub trait GenerationPredicate<T>: private::Sealed {
    fn insert(&self, request: &mut T);
}

impl private::Sealed for () {}

// '()' indicates that we'll use the default values for generation predicates
impl<T> GenerationPredicate<T> for () {
    #[inline]
    fn insert(&self, _request: &mut T) {}
}

macro_rules! define_predicate_types {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            pub struct $name(pub u64);
            impl private::Sealed for $name {}
        )*
    };
}

define_predicate_types! {
    Generation,
    IfGenerationMatches,
    IfGenerationNotMatches,
    IfMetaGenerationMatches,
    IfMetaGenerationNotMatches,
}

macro_rules! impl_generation_predicate {
    (
        $(
            |$self:ident: $pred_ty:ident, $req:ident: $req_ty:ty| $setter:block
        ),* $(,)?
    ) => {
        $(
            impl $crate::generation::GenerationPredicate<$req_ty> for $crate::generation::$pred_ty {
                #[inline]
                fn insert(&$self, $req: &mut $req_ty) {
                    $setter
                }
            }
        )*
    };
    (
        $(
            |$self:ident: $pred_ty:ident, $req:ident: $req_ty:ty| $setter:expr
        ),*
        $(,)?
    ) => {
        $crate::generation::impl_generation_predicate!(
            $(|$self: $pred_ty, $req: $req_ty| { $setter }),*
        );
    };
}

pub(crate) use impl_generation_predicate;

mod private {
    pub trait Sealed {}
}
