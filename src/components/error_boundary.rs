//! Error fallbacks — what to show when the chain isn't talking back.
//!
//! Two surfaces:
//!   - `NodeReconnecting` — stream / RPC silent for a while; data is
//!     stale, not gone. Encourages patience, not panic.
//!   - `NodeError` — explicit failure; the call returned a status,
//!     not a timeout.

use leptos::prelude::*;

use crate::i18n::{t, use_lang};

#[component]
pub fn NodeReconnecting() -> impl IntoView {
    let lang = use_lang();
    view! {
        <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-6 text-center">
            <div class="mx-auto mb-2 flex h-3 w-3">
                <span class="absolute inline-flex h-3 w-3 animate-ping rounded-full bg-amber-400 opacity-75" />
                <span class="relative inline-flex h-3 w-3 rounded-full bg-amber-500" />
            </div>
            <h3 class="mt-2 text-sm font-bold text-amber-300">
                {move || t(lang.get(), "error.node_offline")}
            </h3>
            <p class="mx-auto mt-2 max-w-md text-xs text-zinc-400">
                {move || t(lang.get(), "error.node_body")}
            </p>
        </div>
    }
}

#[component]
pub fn NodeError(message: String) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-rose-500/20 bg-rose-500/5 p-6 text-sm text-rose-200">
            <div class="font-semibold">"Node error"</div>
            <div class="mt-1 font-mono text-xs text-rose-300/80">{message}</div>
        </div>
    }
}
