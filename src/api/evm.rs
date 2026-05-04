//! EVM provider — Ethereum-standard JSON-RPC interface.
//!
//! Two impls live here:
//!
//! - `NoopEvmProvider` — every method returns `Unimplemented`. Used
//!   on SSR where outbound HTTP from the prerender path doesn't make
//!   sense, and as a safety default when an endpoint is unset.
//! - `HttpEvmProvider` (wasm-only) — minimal JSON-RPC client over
//!   `gloo-net`. Covers the read methods the Explorer needs today
//!   (`eth_chainId`, `eth_blockNumber`, `eth_gasPrice`,
//!   `eth_getBalance`, `eth_getCode`, `eth_call`,
//!   `eth_sendRawTransaction`).
//!
//! Why hand-rolled instead of `alloy-providers` / `jsonrpsee`: those
//! pull in ~400 KB gzipped of additional WASM. The handful of methods
//! the UI exercises are flat enough that a 60-line client is more
//! honest than dragging in a full-fat dep.

#[derive(Debug, Clone)]
pub enum EvmError {
    Unimplemented,
    NotFound,
    Rpc(String),
}

/// 20-byte EVM address (H160) as a lowercase 0x-hex string.
pub type EvmAddress = String;

/// 32-byte transaction or block hash (H256) as a lowercase 0x-hex string.
pub type EvmHash = String;

/// Raw return bytes from `eth_call`. ABI decoding lives on the caller.
pub type CallReturn = Vec<u8>;

/// One log entry from `eth_getLogs`. We deliberately keep this as
/// raw hex strings rather than typed Topic/H256 wrappers — the UI
/// renders them as hex anyway, and adding alloy's primitive types
/// would inflate the WASM bundle for a side-feature.
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub block_number: u64,
    pub tx_hash: String,
    pub log_index: u64,
    /// 0–4 indexed topics; topics[0] is the event signature hash.
    pub topics: Vec<String>,
    /// Non-indexed event data, raw 0x-hex.
    pub data: String,
}

pub trait EvmProvider {
    fn chain_id(&self) -> impl std::future::Future<Output = Result<u64, EvmError>>;
    fn block_number(&self) -> impl std::future::Future<Output = Result<u64, EvmError>>;
    fn gas_price(&self) -> impl std::future::Future<Output = Result<u128, EvmError>>;
    fn get_code(
        &self,
        addr: &EvmAddress,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, EvmError>>;
    fn get_balance_wei(
        &self,
        addr: &EvmAddress,
    ) -> impl std::future::Future<Output = Result<u128, EvmError>>;
    fn call(
        &self,
        to: &EvmAddress,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<CallReturn, EvmError>>;
    fn send_raw_transaction(
        &self,
        signed_tx: &[u8],
    ) -> impl std::future::Future<Output = Result<EvmHash, EvmError>>;

    /// `eth_getLogs` for a given address and block window. Caller
    /// passes block numbers as decimal u64; the impl encodes the
    /// JSON-RPC hex form.
    fn get_logs(
        &self,
        addr: &EvmAddress,
        from_block: u64,
        to_block: u64,
    ) -> impl std::future::Future<Output = Result<Vec<LogEntry>, EvmError>>;
}

#[derive(Default, Clone, Copy)]
pub struct NoopEvmProvider;

impl EvmProvider for NoopEvmProvider {
    async fn chain_id(&self) -> Result<u64, EvmError> {
        Err(EvmError::Unimplemented)
    }
    async fn block_number(&self) -> Result<u64, EvmError> {
        Err(EvmError::Unimplemented)
    }
    async fn gas_price(&self) -> Result<u128, EvmError> {
        Err(EvmError::Unimplemented)
    }
    async fn get_code(&self, _addr: &EvmAddress) -> Result<Vec<u8>, EvmError> {
        Err(EvmError::Unimplemented)
    }
    async fn get_balance_wei(&self, _addr: &EvmAddress) -> Result<u128, EvmError> {
        Err(EvmError::Unimplemented)
    }
    async fn call(&self, _to: &EvmAddress, _data: &[u8]) -> Result<CallReturn, EvmError> {
        Err(EvmError::Unimplemented)
    }
    async fn send_raw_transaction(&self, _signed_tx: &[u8]) -> Result<EvmHash, EvmError> {
        Err(EvmError::Unimplemented)
    }
    async fn get_logs(
        &self,
        _addr: &EvmAddress,
        _from: u64,
        _to: u64,
    ) -> Result<Vec<LogEntry>, EvmError> {
        Err(EvmError::Unimplemented)
    }
}

#[cfg(target_arch = "wasm32")]
pub use http::HttpEvmProvider;

#[cfg(target_arch = "wasm32")]
mod http {
    use super::{CallReturn, EvmAddress, EvmError, EvmHash, EvmProvider, LogEntry};
    use gloo_net::http::Request;
    use serde_json::{json, Value};

    /// JSON-RPC over HTTPS. Cheap to construct; one struct per
    /// component is fine. No connection pooling on the browser side
    /// — fetch() reuses HTTP/2 underneath.
    #[derive(Clone)]
    pub struct HttpEvmProvider {
        endpoint: String,
    }

    impl HttpEvmProvider {
        pub fn new(endpoint: impl Into<String>) -> Self {
            Self {
                endpoint: endpoint.into(),
            }
        }

        pub fn default_for_network() -> Self {
            Self::new(crate::config::evm_rpc_endpoint())
        }

        async fn rpc(&self, method: &str, params: Value) -> Result<Value, EvmError> {
            let body = json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": 1,
            });

            let resp = Request::post(&self.endpoint)
                .header("content-type", "application/json")
                .body(body.to_string())
                .map_err(|e| EvmError::Rpc(format!("body: {e}")))?
                .send()
                .await
                .map_err(|e| EvmError::Rpc(format!("send: {e}")))?;

            if !resp.ok() {
                return Err(EvmError::Rpc(format!("http {}", resp.status())));
            }

            let mut json_body: Value = resp
                .json()
                .await
                .map_err(|e| EvmError::Rpc(format!("decode: {e}")))?;

            if let Some(err) = json_body.get("error") {
                let msg = err
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&err.to_string())
                    .to_string();
                return Err(EvmError::Rpc(msg));
            }

            json_body
                .get_mut("result")
                .map(std::mem::take)
                .ok_or_else(|| EvmError::Rpc("no result".into()))
        }

        async fn rpc_hex_int<T: TryFrom<u128>>(
            &self,
            method: &str,
            params: Value,
        ) -> Result<T, EvmError> {
            let v = self.rpc(method, params).await?;
            let s = v.as_str().ok_or_else(|| EvmError::Rpc("not str".into()))?;
            let v = parse_hex_u128(s)?;
            T::try_from(v).map_err(|_| EvmError::Rpc("overflow".into()))
        }
    }

    fn parse_hex_u128(s: &str) -> Result<u128, EvmError> {
        let cleaned = s.strip_prefix("0x").unwrap_or(s);
        if cleaned.is_empty() {
            return Ok(0);
        }
        u128::from_str_radix(cleaned, 16).map_err(|e| EvmError::Rpc(format!("hex: {e}")))
    }

    fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, EvmError> {
        let cleaned = s.strip_prefix("0x").unwrap_or(s);
        if cleaned.is_empty() {
            return Ok(Vec::new());
        }
        hex::decode(cleaned).map_err(|e| EvmError::Rpc(format!("hex: {e}")))
    }

    impl EvmProvider for HttpEvmProvider {
        async fn chain_id(&self) -> Result<u64, EvmError> {
            self.rpc_hex_int("eth_chainId", json!([])).await
        }

        async fn block_number(&self) -> Result<u64, EvmError> {
            self.rpc_hex_int("eth_blockNumber", json!([])).await
        }

        async fn gas_price(&self) -> Result<u128, EvmError> {
            self.rpc_hex_int("eth_gasPrice", json!([])).await
        }

        async fn get_code(&self, addr: &EvmAddress) -> Result<Vec<u8>, EvmError> {
            let v = self.rpc("eth_getCode", json!([addr, "latest"])).await?;
            let s = v.as_str().ok_or_else(|| EvmError::Rpc("not str".into()))?;
            parse_hex_bytes(s)
        }

        async fn get_balance_wei(&self, addr: &EvmAddress) -> Result<u128, EvmError> {
            self.rpc_hex_int("eth_getBalance", json!([addr, "latest"]))
                .await
        }

        async fn call(&self, to: &EvmAddress, data: &[u8]) -> Result<CallReturn, EvmError> {
            let calldata = format!("0x{}", hex::encode(data));
            let v = self
                .rpc(
                    "eth_call",
                    json!([{ "to": to, "data": calldata }, "latest"]),
                )
                .await?;
            let s = v.as_str().ok_or_else(|| EvmError::Rpc("not str".into()))?;
            parse_hex_bytes(s)
        }

        async fn send_raw_transaction(&self, signed: &[u8]) -> Result<EvmHash, EvmError> {
            let body = format!("0x{}", hex::encode(signed));
            let v = self.rpc("eth_sendRawTransaction", json!([body])).await?;
            let s = v.as_str().ok_or_else(|| EvmError::Rpc("not str".into()))?;
            Ok(s.to_string())
        }

        async fn get_logs(
            &self,
            addr: &EvmAddress,
            from_block: u64,
            to_block: u64,
        ) -> Result<Vec<LogEntry>, EvmError> {
            let filter = json!({
                "address": addr,
                "fromBlock": format!("0x{from_block:x}"),
                "toBlock": format!("0x{to_block:x}"),
            });
            let v = self.rpc("eth_getLogs", json!([filter])).await?;
            let arr = v
                .as_array()
                .ok_or_else(|| EvmError::Rpc("logs: not array".into()))?;
            let mut out = Vec::with_capacity(arr.len());
            for entry in arr {
                let block_number = entry
                    .get("blockNumber")
                    .and_then(|v| v.as_str())
                    .and_then(|s| u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok())
                    .unwrap_or(0);
                let log_index = entry
                    .get("logIndex")
                    .and_then(|v| v.as_str())
                    .and_then(|s| u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok())
                    .unwrap_or(0);
                let tx_hash = entry
                    .get("transactionHash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let topics = entry
                    .get("topics")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|t| t.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let data = entry
                    .get("data")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                out.push(LogEntry {
                    block_number,
                    tx_hash,
                    log_index,
                    topics,
                    data,
                });
            }
            Ok(out)
        }
    }
}
