//! Thin wrapper around the generated tonic client over a gRPC-Web transport.
//!
//! Why a wrapper instead of using `SentrixClient` directly:
//!   1. Centralises the endpoint URL (`crate::GRPC_ENDPOINT`).
//!   2. Hides the `tonic_web_wasm_client::Client` type so screens don't
//!      need to know which transport they're on.
//!   3. Gives us a single place to attach interceptors (auth, telemetry)
//!      when we add them.

use tonic_web_wasm_client::Client as WebClient;

use super::pb::{
    get_block_request::Selector, sentrix_client::SentrixClient, BlockHeight, EventFilter,
    GetBalanceRequest, GetBlockRequest, GetMempoolRequest, GetSupplyRequest,
    GetValidatorSetRequest, Mempool, StreamEventsRequest, Supply, ValidatorSet,
};

/// The concrete client type after we've wired the wasm transport.
pub type Inner = SentrixClient<WebClient>;

#[derive(Clone)]
pub struct SentrixGrpcClient {
    inner: Inner,
}

impl SentrixGrpcClient {
    /// Build a client targeting the given endpoint. Cheap; just constructs
    /// a `fetch()` wrapper. No network IO until the first RPC.
    pub fn new(endpoint: impl Into<String>) -> Self {
        let transport = WebClient::new(endpoint.into());
        Self {
            inner: SentrixClient::new(transport),
        }
    }

    /// `latest` selector — matches the `GetBlock { latest: true }` path on
    /// the chain-side handler.
    pub async fn get_latest_block(&mut self) -> Result<super::pb::Block, tonic::Status> {
        let req = GetBlockRequest {
            selector: Some(Selector::Latest(true)),
        };
        let resp = self.inner.get_block(req).await?;
        Ok(resp.into_inner())
    }

    pub async fn get_block_by_height(
        &mut self,
        height: u64,
    ) -> Result<super::pb::Block, tonic::Status> {
        let req = GetBlockRequest {
            selector: Some(Selector::Height(BlockHeight { value: height })),
        };
        let resp = self.inner.get_block(req).await?;
        Ok(resp.into_inner())
    }

    pub async fn get_balance(
        &mut self,
        addr: [u8; 20],
    ) -> Result<super::pb::Account, tonic::Status> {
        let req = GetBalanceRequest {
            address: Some(super::pb::Address {
                value: addr.to_vec(),
            }),
            at_height: None,
        };
        let resp = self.inner.get_balance(req).await?;
        Ok(resp.into_inner())
    }

    /// v0.4 — full active set + jail/active flags + per-validator stake.
    pub async fn get_validator_set(&mut self) -> Result<ValidatorSet, tonic::Status> {
        let req = GetValidatorSetRequest { at_height: None };
        let resp = self.inner.get_validator_set(req).await?;
        Ok(resp.into_inner())
    }

    /// v0.4 — minted/burned/circulating supply snapshot.
    pub async fn get_supply(&mut self) -> Result<Supply, tonic::Status> {
        let req = GetSupplyRequest { at_height: None };
        let resp = self.inner.get_supply(req).await?;
        Ok(resp.into_inner())
    }

    /// v0.4 — pending-tx size + capped header window. `limit = 0` ⇒
    /// server default (100). Pass a smaller limit for the dashboard
    /// header card (just need `size`); pass max 500 for the mempool
    /// panel that lists actual entries.
    pub async fn get_mempool(&mut self, limit: u32) -> Result<Mempool, tonic::Status> {
        let req = GetMempoolRequest { limit };
        let resp = self.inner.get_mempool(req).await?;
        Ok(resp.into_inner())
    }

    /// Server-streaming events. Returns a `Streaming<ChainEvent>` that the
    /// caller drains with `.message().await`. Filter list is sent verbatim;
    /// empty = subscribe-all (server-side filter).
    pub async fn subscribe_events(
        &mut self,
        filters: Vec<EventFilter>,
    ) -> Result<tonic::Streaming<super::pb::ChainEvent>, tonic::Status> {
        let req = StreamEventsRequest {
            filters: filters.into_iter().map(|f| f as i32).collect(),
            from_sequence: 0,
        };
        let resp = self.inner.stream_events(req).await?;
        Ok(resp.into_inner())
    }
}

/// Convenience: short-form hex of a 32-byte hash for UI rendering.
/// Returns "—" on length mismatch rather than panicking; the gRPC
/// contract guarantees 32 bytes but we're playing defence.
pub fn hash_short(h: &super::pb::Hash) -> String {
    if h.value.len() != 32 {
        return "—".into();
    }
    let hex = hex::encode(&h.value);
    format!("{}…{}", &hex[..6], &hex[hex.len() - 4..])
}
