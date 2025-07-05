#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct EmulatorProvider;

impl super::BaseTokenProvider for EmulatorProvider {
    #[inline]
    fn name(&self) -> &'static str {
        "emulator"
    }
}

impl super::TokenProvider for EmulatorProvider {
    #[inline]
    fn get_token(&self) -> crate::GetTokenFuture<'_> {
        crate::GetTokenFuture::new_emulator()
    }
}
