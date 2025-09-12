use gcp_auth_channel::AuthChannel;
use prost::bytes::BytesMut;
use prost::{DecodeError, Message};
use protos::longrunning::operations_client::OperationsClient;
use protos::longrunning::{
    self, CancelOperationRequest, GetOperationRequest, Operation, WaitOperationRequest,
};
use protos::{protobuf, rpc};
use timestamp::Duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Status(#[from] tonic::Status),
}

impl From<rpc::Status> for Error {
    fn from(value: rpc::Status) -> Self {
        let code = tonic::Code::from_i32(value.code);
        let message = value.message;

        if value.details.is_empty() {
            Self::Status(tonic::Status::new(code, message))
        } else {
            let capacity = value
                .details
                .iter()
                .map(|detail| detail.encoded_len())
                .sum::<usize>();

            let mut dst = BytesMut::with_capacity(capacity + value.details.len());

            for detail in value.details.iter() {
                if detail.encode(&mut dst).is_err() {
                    break;
                }
            }

            Self::Status(tonic::Status::with_details(code, message, dst.freeze()))
        }
    }
}

impl From<&rpc::Status> for Error {
    fn from(value: &rpc::Status) -> Self {
        let code = tonic::Code::from_i32(value.code);
        let message = value.message.clone();

        if value.details.is_empty() {
            Self::Status(tonic::Status::new(code, message))
        } else {
            let capacity = value
                .details
                .iter()
                .map(|detail| detail.encoded_len())
                .sum::<usize>();

            let mut dst = BytesMut::with_capacity(capacity + value.details.len());

            for detail in value.details.iter() {
                if detail.encode(&mut dst).is_err() {
                    break;
                }
            }

            Self::Status(tonic::Status::with_details(code, message, dst.freeze()))
        }
    }
}

pub struct OperationHandle<Meta = protobuf::Any, Response = protobuf::Any> {
    operation: Operation,
    // potentially set with each poll, even if the operation is __not__ done.
    metadata: Option<Meta>,
    // Only set when 'operation.done == true', and only if the resulting union field is not an
    // error status.
    response: Option<Response>,
    channel: AuthChannel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WaitStatus {
    Finished,
    TimedOut,
}

impl<Meta, Response> OperationHandle<Meta, Response> {
    pub fn from_channel(channel: AuthChannel, operation: Operation) -> Self {
        Self {
            operation,
            metadata: None,
            response: None,
            channel,
        }
    }

    pub fn metadata(&self) -> Option<&Meta> {
        self.metadata.as_ref()
    }

    pub fn is_done(&self) -> bool {
        self.operation.done
    }

    pub fn result(&self) -> Option<&Response> {
        self.response.as_ref()
    }

    pub fn operation(&self) -> &Operation {
        &self.operation
    }

    pub fn into_result(mut self) -> Result<Response, Error>
    where
        Response: Message + Default,
    {
        assert!(
            self.is_done(),
            "operation has not been polled to completion"
        );

        self.parse_result()?;

        Ok(self.response.unwrap())
    }

    pub async fn cancel(self) -> Result<(), Error> {
        let request = CancelOperationRequest {
            name: self.operation.name,
        };

        OperationsClient::new(self.channel.clone())
            .cancel_operation(request)
            .await?;

        Ok(())
    }

    pub async fn poll_operation(&mut self) -> Result<(), Error> {
        let request = GetOperationRequest {
            name: self.operation.name.clone(),
        };

        let new_status = self.client().get_operation(request).await?.into_inner();

        self.handle_new_state(new_status);

        Ok(())
    }

    pub async fn wait(mut self, timeout: Option<Duration>) -> Result<Response, Error>
    where
        Response: Message + Default,
    {
        while !self.is_done() {
            if let Err(error) = self.wait_once(timeout).await {
                println!("{error:#?}");
                return Err(error.into());
            }
        }

        let result = self
            .operation
            .result
            .expect("if we're done, the result should be set");

        let resp = match result {
            longrunning::operation::Result::Response(resp) => resp,
            longrunning::operation::Result::Error(status) => return Err(Error::from(status)),
        };

        Response::decode(resp.value).map_err(Error::from)
    }

    async fn wait_once(&mut self, timeout: Option<Duration>) -> Result<(), tonic::Status> {
        let request = WaitOperationRequest {
            name: self.operation.name.clone(),
            timeout: timeout.map(Duration::into),
        };

        match self.client().wait_operation(request).await {
            Ok(new) => {
                self.handle_new_state(new.into_inner());
                Ok(())
            }
            Err(other) => Err(other),
        }
    }

    fn handle_new_state(&mut self, new: Operation) {
        if self.operation.metadata != new.metadata {
            self.metadata = None;
        }

        if self.operation.result != new.result || self.operation.done != new.done {
            self.response = None;
        }

        self.operation = new;
    }

    fn client(&self) -> OperationsClient<AuthChannel> {
        OperationsClient::new(self.channel.clone())
    }

    pub fn parse_metadata(&mut self) -> Result<Option<&Meta>, Error>
    where
        Meta: prost::Message + Default,
    {
        if let Some(ref meta) = self.metadata {
            return Ok(Some(meta));
        }

        match self.operation.metadata {
            Some(ref raw_meta) => {
                let decoded = Meta::decode(raw_meta.value.clone())?;
                Ok(Some(self.metadata.insert(decoded)))
            }
            None => return Ok(None),
        }
    }

    pub fn parse_result(&mut self) -> Result<Option<&Response>, Error>
    where
        Response: prost::Message + Default,
    {
        match self.response {
            Some(ref resp) => return Ok(Some(resp)),
            None if !self.is_done() => return Ok(None),
            _ => (),
        }

        match self.operation.result {
            None => Ok(None),
            Some(longrunning::operation::Result::Error(ref status)) => Err(Error::from(status)),
            Some(longrunning::operation::Result::Response(ref resp)) => {
                let decoded = Response::decode(resp.value.clone())?;
                Ok(Some(self.response.insert(decoded)))
            }
        }
    }
}
