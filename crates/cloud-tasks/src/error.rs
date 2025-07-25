use gcp_auth_provider::channel::ChannelError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] gcp_auth_provider::Error),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    MissingProtoField(#[from] MissingProtoField),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

impl From<ChannelError> for Error {
    fn from(value: ChannelError) -> Self {
        match value {
            ChannelError::Auth(auth) => Self::Auth(auth),
            ChannelError::Transport(transport) => Self::Transport(transport),
        }
    }
}

impl Error {
    pub(crate) fn missing_proto_field<T: ?Sized>(field_name: &'static str) -> Self {
        Self::MissingProtoField(MissingProtoField {
            on_type: std::any::type_name::<T>(),
            field_name,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{on_type} expected field {field_name}")]
pub struct MissingProtoField {
    on_type: &'static str,
    field_name: &'static str,
}
