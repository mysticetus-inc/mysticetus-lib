pub struct CommitToken(pub(super) String);

impl CommitToken {
    fn from_stream<S: AsRef<str> + Into<String>>(s: S) -> Option<Self> {
        // defaults streams dont need to be committed
        if s.as_ref().ends_with("_default") {
            None
        } else {
            Some(Self(s.into()))
        }
    }
}
