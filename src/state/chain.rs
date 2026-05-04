//! Chain metadata — fetched once at startup and cached for the
//! session. `eth_chainId` doesn't drift mid-session, so a single
//! lazy-fetch is enough.

use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct ChainState {
    pub chain_id: ReadSignal<Option<u64>>,
}

pub fn provide_chain_state() {
    let (chain_id, set_chain_id) = signal::<Option<u64>>(None);

    #[cfg(target_arch = "wasm32")]
    {
        use crate::api::evm::{EvmProvider, HttpEvmProvider};
        leptos::task::spawn_local(async move {
            // One-shot — no loop. If the call fails the slot stays
            // None and the UI shows "—" rather than blocking.
            let p = HttpEvmProvider::default_for_network();
            if let Ok(id) = p.chain_id().await {
                set_chain_id.set(Some(id));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = set_chain_id;

    provide_context(ChainState { chain_id });
}
