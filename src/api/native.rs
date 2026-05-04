//! Native provider — wraps the existing `SentrixGrpcClient`.
//!
//! Trait-shaped so screens can program against `dyn NativeProvider`
//! without depending on the proto types directly. Real impl is
//! `SentrixNativeProvider`; tests can swap a stub.

use crate::grpc::client::SentrixGrpcClient;
use crate::grpc::pb;
use crate::GRPC_ENDPOINT;

#[derive(Debug, Clone)]
pub enum NativeError {
    NotFound,
    Rpc(String),
}

impl From<tonic::Status> for NativeError {
    fn from(s: tonic::Status) -> Self {
        match s.code() {
            tonic::Code::NotFound => Self::NotFound,
            _ => Self::Rpc(s.message().to_string()),
        }
    }
}

pub trait NativeProvider {
    /// `latest` selector — same semantics as `eth_blockNumber` +
    /// `eth_getBlockByNumber("latest")` collapsed.
    fn get_latest_block(&self)
        -> impl std::future::Future<Output = Result<pb::Block, NativeError>>;

    fn get_block_by_height(
        &self,
        height: u64,
    ) -> impl std::future::Future<Output = Result<pb::Block, NativeError>>;

    fn get_balance(
        &self,
        addr: [u8; 20],
    ) -> impl std::future::Future<Output = Result<pb::Account, NativeError>>;
}

#[derive(Clone)]
pub struct SentrixNativeProvider {
    endpoint: String,
}

impl SentrixNativeProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    /// Default constructor pointing at the network the binary was
    /// compiled for.
    pub fn default_for_network() -> Self {
        Self::new(GRPC_ENDPOINT)
    }

    fn client(&self) -> SentrixGrpcClient {
        SentrixGrpcClient::new(&self.endpoint)
    }
}

impl NativeProvider for SentrixNativeProvider {
    async fn get_latest_block(&self) -> Result<pb::Block, NativeError> {
        let mut c = self.client();
        c.get_latest_block().await.map_err(Into::into)
    }

    async fn get_block_by_height(&self, height: u64) -> Result<pb::Block, NativeError> {
        let mut c = self.client();
        c.get_block_by_height(height).await.map_err(Into::into)
    }

    async fn get_balance(&self, addr: [u8; 20]) -> Result<pb::Account, NativeError> {
        let mut c = self.client();
        c.get_balance(addr).await.map_err(Into::into)
    }
}
