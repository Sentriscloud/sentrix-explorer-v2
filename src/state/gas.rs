//! Gas price polling — `eth_gasPrice` over the EVM RPC every 6 s.
//!
//! Lifted into shared state so any panel that wants the number reads
//! the same signal. SSR side keeps the signal in the `None` state
//! since the HTTP provider is wasm-only.

use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
const HISTORY_LEN: usize = 60; // 60 × 6 s = 6 min rolling window

#[derive(Clone, Copy)]
pub struct GasPriceState {
    pub gwei: ReadSignal<Option<f64>>,
    pub history: ReadSignal<Vec<f64>>,
}

pub fn provide_gas_price() {
    let (gwei, set_gwei) = signal::<Option<f64>>(None);
    let (history, set_history) = signal::<Vec<f64>>(Vec::new());

    #[cfg(target_arch = "wasm32")]
    {
        use crate::api::evm::{EvmProvider, HttpEvmProvider};
        leptos::task::spawn_local(async move {
            let p = HttpEvmProvider::default_for_network();
            // Exponential backoff on consecutive failures — 6 s →
            // 12 → 24 → 48 → cap 60. Resets to 6 on success. Keeps
            // us from hammering a 429-throttled or briefly-down RPC.
            const BASE_MS: i32 = 6_000;
            const MAX_MS: i32 = 60_000;
            let mut delay = BASE_MS;
            loop {
                match p.gas_price().await {
                    Ok(wei) => {
                        let g = wei as f64 / 1_000_000_000.0;
                        set_gwei.set(Some(g));
                        set_history.update(|list| {
                            list.push(g);
                            if list.len() > HISTORY_LEN {
                                let drop = list.len() - HISTORY_LEN;
                                list.drain(..drop);
                            }
                        });
                        delay = BASE_MS;
                    }
                    Err(_) => {
                        delay = (delay.saturating_mul(2)).min(MAX_MS);
                    }
                }
                crate::util::sleep_ms(delay).await;
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (set_gwei, set_history);
    }

    provide_context(GasPriceState { gwei, history });
}
