//! gRPC-Web client glue. The proto-generated module lives at `pb::`;
//! everything UI-facing goes through `client::SentrixGrpcClient`.

/// Generated tonic + prost types for the `sentrix.v1` schema. Re-export
/// of the `sentrix-proto` crate published from the chain repo, so this
/// app stays in sync with the server schema without vendoring its own
/// copy.
pub mod pb {
    pub use sentrix_proto::*;
}

pub mod client;
