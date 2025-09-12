/// Helper macro for implementing [tower::Service] for both Self, and &Self when the inner service
/// implements tower::Service by ref. (i.e &'a Svc: tower::Service)
///
/// poll_ready just defers to the inner service,
/// and call is given access to 'self' and the request, which is assumed to implement
/// crate::http_svc::HttpRequest.
macro_rules! impl_service_for_wrapper_and_ref {
    (
        $t:ident:: <
        $svc_ty:ident > ::
        $svc_field:ident { call: | $self_:ident, $req:ident : $req_ty:ident | $b:block }
    ) => {
        impl<$svc_ty, $req_ty> ::tower::Service<$req_ty> for $t<$svc_ty>
        where
            $req_ty: $crate::http_svc::HttpRequest,
            $svc_ty: ::tower::Service<$req_ty>,
        {
            type Error = <$svc_ty>::Error;
            type Future = <$svc_ty>::Future;
            type Response = <$svc_ty>::Response;

            #[inline]
            fn poll_ready(
                &mut self,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                self.$svc_field.poll_ready(cx)
            }

            #[inline]
            fn call($self_: &mut Self, mut $req: $req_ty) -> Self::Future {
                {
                    $b
                };

                $self_.$svc_field.call($req)
            }
        }

        impl<'a, $svc_ty, $req_ty> ::tower::Service<$req_ty> for &'a $t<$svc_ty>
        where
            $req_ty: $crate::http_svc::HttpRequest,
            &'a $svc_ty: ::tower::Service<$req_ty>,
        {
            type Error = <&'a $svc_ty as ::tower::Service<$req_ty>>::Error;
            type Future = <&'a $svc_ty as ::tower::Service<$req_ty>>::Future;
            type Response = <&'a $svc_ty as ::tower::Service<$req_ty>>::Response;

            #[inline]
            fn poll_ready(
                &mut self,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                (&mut &self.$svc_field).poll_ready(cx)
            }

            #[inline]
            fn call($self_: &mut Self, mut $req: $req_ty) -> Self::Future {
                {
                    $b
                };

                (&mut &$self_.$svc_field).call($req)
            }
        }
    };
    (
        $t:ident:: <
        $svc_ty:ident > ::
        $svc_field:ident { call: | $self_:ident, $req:ident : $req_ty:ident | $b:expr }
    ) => {
        $crate::util::impl_service_for_wrapper_and_ref! {
            $t :: < $svc_ty > :: $svc_field {
                call: | $self_, $req : $req_ty | { $b }
            }
        }
    };
}

pub(crate) use impl_service_for_wrapper_and_ref;
