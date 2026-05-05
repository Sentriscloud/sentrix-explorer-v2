//! Shared block-feed state — one gRPC subscription, many consumers.
//!
//! `LiveBlockFeed` renders the rows; `StatsPanel` computes a rolling-
//! window TPS from the same vec. Lifting the signal here means we only
//! spin up a single `subscribe_events` (or polling fallback) per
//! session, no matter how many widgets read from it.
//!
//! ## Streaming → poll fallback
//!
//! Chain v0.2 `StreamEvents` returns `Status::Unimplemented`; we fall
//! back to a 2 s `GetBlock(latest)` poll. The instant chain ships v0.3
//! with `BlockFinalized` events firing, the streaming arm takes over —
//! no consumer-side change.

use leptos::prelude::*;

use crate::grpc::pb::{Block, Transaction};

#[cfg(target_arch = "wasm32")]
const MAX_FEED_LEN: usize = 25;
#[cfg(target_arch = "wasm32")]
const MAX_TX_INDEX: usize = 500;

/// Plain-Rust shape used by the view layer. Decoupled from the proto
/// `Block` so a future field shuffle in the proto doesn't ripple.
///
/// `hash_hex` is the full lowercase 64-char hex; truncation happens in
/// the view. Keying `<For/>` on the full hash makes us robust to brief
/// fork moments where two distinct blocks could share a height.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockRow {
    pub height: u64,
    pub hash_hex: String,
    pub tx_count: usize,
    pub timestamp: u64,
    /// 20-byte proposer address as lowercase hex (no 0x prefix).
    /// Empty string when the block carries no proposer field.
    pub proposer_hex: String,
}

impl From<&Block> for BlockRow {
    fn from(b: &Block) -> Self {
        Self {
            height: b.index,
            hash_hex: b
                .hash
                .as_ref()
                .map(|h| hex::encode(&h.value))
                .unwrap_or_else(|| "0".repeat(64)),
            tx_count: b.transactions.len(),
            timestamp: b.timestamp,
            proposer_hex: b
                .proposer
                .as_ref()
                .map(|a| hex::encode(&a.value))
                .unwrap_or_default(),
        }
    }
}

/// One observed transaction kept in the rolling cache. `tx` is the
/// proto-shaped struct so detail screens can decode any field they
/// need without us pre-flattening at index time.
#[derive(Clone, Debug, PartialEq)]
pub struct IndexedTx {
    pub tx: Transaction,
    pub block_height: u64,
    pub block_hash_hex: String,
}

/// Context value handed to every consumer. Read-only signals — only
/// the producer in `provide_block_feed` writes.
#[derive(Clone, Copy)]
pub struct BlockFeedState {
    pub blocks: ReadSignal<Vec<BlockRow>>,
    pub status: ReadSignal<&'static str>,
    /// Rolling cache of recently-observed txs, newest first. Capped
    /// at `MAX_TX_INDEX`. Detail screens look up by txid here.
    pub tx_index: ReadSignal<Vec<IndexedTx>>,
}

/// Spawn the gRPC subscription (or poll fallback) and provide the
/// resulting signals via context. Call once near the route root.
pub fn provide_block_feed() {
    let (blocks, set_blocks) = signal(Vec::<BlockRow>::new());
    let (tx_index, set_tx_index) = signal(Vec::<IndexedTx>::new());
    let (status, set_status) = signal::<&'static str>("connecting…");

    // Producer is wasm-only — SSR pre-render mounts the components,
    // signals stay empty, and the hydrated bundle picks up the live
    // subscription. Calling `spawn_local` outside a LocalSet on the
    // tokio multi-thread runtime panics, so we don't.
    #[cfg(target_arch = "wasm32")]
    {
        use crate::grpc::{
            client::SentrixGrpcClient,
            pb::{chain_event::Event as PbEvent, EventFilter},
        };
        use crate::GRPC_ENDPOINT;
        leptos::task::spawn_local(async move {
            let mut client = SentrixGrpcClient::new(GRPC_ENDPOINT);
            match client
                .subscribe_events(vec![EventFilter::BlockFinalized])
                .await
            {
                Ok(mut stream) => {
                    set_status.set("live · streaming");
                    loop {
                        match stream.message().await {
                            Ok(Some(ev)) => {
                                if let Some(PbEvent::BlockFinalized(bf)) = ev.event {
                                    if let Some(block) = bf.block.as_ref() {
                                        push_block(set_blocks, BlockRow::from(block));
                                        index_txs(set_tx_index, block);
                                    }
                                }
                            }
                            Ok(None) => {
                                set_status.set("stream closed · retrying via poll");
                                break;
                            }
                            Err(_) => {
                                set_status.set("stream error · falling back to poll");
                                break;
                            }
                        }
                    }
                }
                Err(s) if s.code() == tonic::Code::Unimplemented => {
                    set_status.set("polling · stream not yet on chain");
                }
                Err(_) => {
                    set_status.set("rpc error · falling back to poll");
                }
            }
            let mut last_height: Option<u64> = None;
            loop {
                crate::util::sleep_ms(2_000).await;
                match client.get_latest_block().await {
                    Ok(block) => {
                        let row = BlockRow::from(&block);
                        if Some(row.height) != last_height {
                            last_height = Some(row.height);
                            push_block(set_blocks, row);
                            index_txs(set_tx_index, &block);
                            set_status.set("live · polling");
                        }
                    }
                    Err(_) => {
                        set_status.set("rpc error · retrying");
                    }
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Bind set_blocks/set_tx_index/set_status so the SSR build
        // doesn't lint them as unused.
        let _ = (set_blocks, set_tx_index, set_status);
    }

    provide_context(BlockFeedState {
        blocks,
        status,
        tx_index,
    });
}

#[cfg(target_arch = "wasm32")]
fn index_txs(set: WriteSignal<Vec<IndexedTx>>, block: &Block) {
    let block_height = block.index;
    let block_hash_hex = block
        .hash
        .as_ref()
        .map(|h| hex::encode(&h.value))
        .unwrap_or_default();
    let entries: Vec<IndexedTx> = block
        .transactions
        .iter()
        .map(|tx| IndexedTx {
            tx: tx.clone(),
            block_height,
            block_hash_hex: block_hash_hex.clone(),
        })
        .collect();
    if entries.is_empty() {
        return;
    }
    set.update(|list| {
        // Newest at the front; capped from the back.
        for e in entries.into_iter().rev() {
            list.insert(0, e);
        }
        if list.len() > MAX_TX_INDEX {
            list.truncate(MAX_TX_INDEX);
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn push_block(set_blocks: WriteSignal<Vec<BlockRow>>, row: BlockRow) {
    set_blocks.update(|list| {
        // Dedupe on hash — height alone collides during a brief fork.
        if list.first().map(|r| r.hash_hex.as_str()) == Some(row.hash_hex.as_str()) {
            return;
        }
        list.insert(0, row);
        if list.len() > MAX_FEED_LEN {
            list.truncate(MAX_FEED_LEN);
        }
    });
}
