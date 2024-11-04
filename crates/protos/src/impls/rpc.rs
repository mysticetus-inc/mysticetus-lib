use tonic::Status;

use crate::protos::google::rpc;

impl From<rpc::Status> for Status {
    fn from(rpc_status: rpc::Status) -> Self {
        let tonic_code = tonic::Code::from_i32(rpc_status.code);

        match serde_json::to_vec(&rpc_status.details) {
            Ok(bytes) => Status::with_details(tonic_code, rpc_status.message, bytes.into()),
            // ignore any serialization errors, and just return with empty details
            Err(_) => Status::new(tonic_code, rpc_status.message),
        }
    }
}
