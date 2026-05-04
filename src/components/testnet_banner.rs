use leptos::prelude::*;

use crate::config::Network;

/// Top-of-page strip rendered only on testnet builds. Compile-time
/// gated via `Network::current()` so mainnet bundles never carry the
/// banner markup at all (smaller WASM, zero risk of stray testnet
/// chrome leaking onto a mainnet artifact through a render bug).
#[component]
pub fn TestnetBanner() -> impl IntoView {
    if Network::current() != Network::Testnet {
        return None;
    }

    Some(view! {
        <div class="border-b border-amber-500/20 bg-amber-500/10 py-1.5 text-center">
            <span class="text-[11px] font-bold uppercase tracking-[0.18em] text-amber-400">
                "⚠ Sentrix Testnet — assets have no real-world value"
            </span>
        </div>
    })
}
