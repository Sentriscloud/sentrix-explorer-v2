//! gRPC-Web client glue. The proto-generated module lives at `pb::`;
//! everything UI-facing goes through `client::SentrixGrpcClient`.

#[allow(clippy::all, clippy::pedantic, dead_code)]
pub mod pb {
    tonic::include_proto!("sentrix.v1");
}

pub mod client;
