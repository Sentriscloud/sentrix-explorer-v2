//! Mempool state — pending transactions streamed from gRPC.
//!
//! Subscribes to the `PendingTx` event filter on `StreamEvents`. Chain
//! v0.2 returns `Status::Unimplemented` for the whole stream method;
//! the empty state stays visible until v0.3 ships, then the rows fill
//! in automatically.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::grpc::{
    client::SentrixGrpcClient,
    pb::{chain_event::Event as PbEvent, EventFilter, Transaction},
};
use crate::GRPC_ENDPOINT;

const MAX_PENDING: usize = 20;

#[derive(Clone, Debug)]
pub struct PendingTxRow {
    pub txid_hex: String,
    pub from_hex: String,
    pub to_hex: String,
    pub amount_sentri: u64,
    pub fee_sentri: u64,
    pub tx_type: u32,
}

impl From<&Transaction> for PendingTxRow {
    fn from(t: &Transaction) -> Self {
        let txid_hex = t
            .txid
            .as_ref()
            .map(|h| hex::encode(&h.value))
            .unwrap_or_else(|| "0".repeat(64));
        let from_hex = t
            .from_address
            .as_ref()
            .map(|a| hex::encode(&a.value))
            .unwrap_or_else(|| "0".repeat(40));
        let to_hex = t
            .to_address
            .as_ref()
            .map(|a| hex::encode(&a.value))
            .unwrap_or_else(|| "0".repeat(40));
        Self {
            txid_hex,
            from_hex,
            to_hex,
            amount_sentri: t.amount.as_ref().map(|a| a.sentri).unwrap_or(0),
            fee_sentri: t.fee.as_ref().map(|a| a.sentri).unwrap_or(0),
            tx_type: t.tx_type,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MempoolState {
    pub pending: ReadSignal<Vec<PendingTxRow>>,
    pub status: ReadSignal<&'static str>,
}

pub fn provide_mempool() {
    let (pending, set_pending) = signal(Vec::<PendingTxRow>::new());
    let (status, set_status) = signal::<&'static str>("connecting…");

    spawn_local(async move {
        let mut client = SentrixGrpcClient::new(GRPC_ENDPOINT);

        match client.subscribe_events(vec![EventFilter::PendingTx]).await {
            Ok(mut stream) => {
                set_status.set("live · streaming");
                loop {
                    match stream.message().await {
                        Ok(Some(ev)) => {
                            if let Some(PbEvent::PendingTx(p)) = ev.event {
                                if let Some(tx) = p.tx.as_ref() {
                                    push_pending(set_pending, PendingTxRow::from(tx));
                                }
                            }
                        }
                        Ok(None) | Err(_) => {
                            set_status.set("stream closed");
                            break;
                        }
                    }
                }
            }
            Err(s) if s.code() == tonic::Code::Unimplemented => {
                set_status.set("awaiting chain v0.3 stream");
            }
            Err(_) => {
                set_status.set("rpc error");
            }
        }
    });

    provide_context(MempoolState { pending, status });
}

fn push_pending(set: WriteSignal<Vec<PendingTxRow>>, row: PendingTxRow) {
    set.update(|list| {
        if list.iter().any(|r| r.txid_hex == row.txid_hex) {
            return;
        }
        list.insert(0, row);
        if list.len() > MAX_PENDING {
            list.truncate(MAX_PENDING);
        }
    });
}
