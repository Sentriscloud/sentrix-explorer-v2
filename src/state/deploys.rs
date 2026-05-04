//! Deploy history — track contracts deployed via `/lab` so the
//! `/contracts` page can list them.
//!
//! Persisted to `localStorage["sentrix-deploys"]` as a JSON array,
//! capped at the last 50 entries. Per-network keying isn't applied
//! today because the binary already only sees one network at build
//! time, so the list is implicitly scoped.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "sentrix-deploys";
const MAX_ENTRIES: usize = 50;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeployRecord {
    pub tx_hash: String,
    /// Source byte length — gives the user a rough size signal
    /// without needing to decode the bytecode.
    pub bytecode_len: usize,
    /// Wall-clock millis at deploy submission. Browser side via
    /// `Date.now()`. Used purely for relative sorting + display.
    pub submitted_ms: u64,
    /// Connected wallet address that submitted the deploy. Useful
    /// for filtering when the operator switches wallets mid-session.
    pub from: String,
}

#[derive(Clone, Copy)]
pub struct DeployHistoryState {
    pub deploys: ReadSignal<Vec<DeployRecord>>,
    pub append: WriteSignal<Vec<DeployRecord>>,
}

pub fn provide_deploy_history() {
    let initial = read_persisted();
    let (deploys, set_deploys) = signal(initial);
    provide_context(DeployHistoryState {
        deploys,
        append: set_deploys,
    });
}

/// Push a record to the history and persist. Newest first; capped.
pub fn record_deploy(state: DeployHistoryState, record: DeployRecord) {
    state.append.update(|list| {
        list.insert(0, record);
        if list.len() > MAX_ENTRIES {
            list.truncate(MAX_ENTRIES);
        }
        persist(list);
    });
}

#[cfg(target_arch = "wasm32")]
fn read_persisted() -> Vec<DeployRecord> {
    let Some(win) = web_sys::window() else {
        return Vec::new();
    };
    let raw = win
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten());
    match raw {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => Vec::new(),
    }
}

#[cfg(target_arch = "wasm32")]
fn persist(list: &[DeployRecord]) {
    let Some(win) = web_sys::window() else { return };
    let Ok(Some(storage)) = win.local_storage() else {
        return;
    };
    if let Ok(json) = serde_json::to_string(list) {
        let _ = storage.set_item(STORAGE_KEY, &json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_persisted() -> Vec<DeployRecord> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn persist(_list: &[DeployRecord]) {}

#[cfg(target_arch = "wasm32")]
pub fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_ms() -> u64 {
    0
}
