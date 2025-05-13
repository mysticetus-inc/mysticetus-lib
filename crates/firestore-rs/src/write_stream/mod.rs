use bytes::Bytes;
use net_utils::bidi2::{self, RequestSink};
use protos::firestore::{Write, WriteRequest, WriteResponse, WriteResult};

use crate::doc::WriteBuilder;
use crate::{Firestore, PathComponent};

pub struct WriteStream {
    firestore: Firestore,
    sink: RequestSink<WriteRequest>,
    responses: tonic::Streaming<WriteResponse>,
    stream_token: Bytes,
    stream_id: Box<str>,
}

impl WriteStream {
    pub(crate) async fn new(mut firestore: Firestore) -> crate::Result<Self> {
        let (sink, responses, first_response) =
            create_stream(&mut firestore, String::new(), Bytes::new()).await?;

        Ok(Self {
            firestore,
            sink,
            responses,
            stream_token: first_response.stream_token,
            stream_id: first_response.stream_id.into_boxed_str(),
        })
    }

    pub async fn write_one<D, C, W>(
        &mut self,
        wb: WriteBuilder<'_, C, D, W>,
    ) -> crate::Result<WriteResult>
    where
        C: PathComponent,
        D: PathComponent,
        W: crate::doc::write_type::WriteType,
    {
        let (_, write) = wb.into_parts();
        todo!()
    }
}

async fn create_stream(
    firestore: &mut Firestore,
    stream_id: String,
    stream_token: Bytes,
) -> crate::Result<(
    RequestSink<WriteRequest>,
    tonic::Streaming<WriteResponse>,
    WriteResponse,
)> {
    let database = firestore.qualified_db_path().to_owned();
    let mut client = firestore.client.get_mut_ref();

    let (sink, stream) = bidi2::build_pair();

    let first_request = WriteRequest {
        database,
        stream_id,
        stream_token,
        writes: Vec::new(),
        labels: Default::default(),
    };

    sink.send(first_request)
        .expect("stream hasnt even been used yet, the inner channel is alive");

    let mut responses = client.write(stream).await?.into_inner();

    let Some(first_resp) = responses.message().await? else {
        return Err(crate::Error::Internal(
            "firestore.Write dropped the stream without responding",
        ));
    };

    Ok((sink, responses, first_resp))
}
