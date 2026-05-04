//! `/contracts` — local deploy history.
//!
//! Pure browser-side: pulls from the `DeployHistoryState` context
//! which itself reads `localStorage`. The chain doesn't keep a
//! deploy-history index, so this is per-device. When/if a global
//! verification registry ships, this page can union the local cache
//! with the remote registry.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::identicon::Identicon;
use crate::i18n::{t, use_lang};
use crate::state::deploys::{DeployHistoryState, DeployRecord};

#[component]
pub fn ContractsScreen() -> impl IntoView {
    let history = use_context::<DeployHistoryState>().expect("DeployHistoryState context");
    let lang = use_lang();

    view! {
        <Title text="Sentrix Contracts — Obsidian Engine" />

        <section class="glass-card rounded-2xl p-6">
            <header class="mb-6 flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-bold italic tracking-tighter text-zinc-100">
                        {move || t(lang.get(), "contracts.heading")}
                    </h2>
                    <p class="mt-1 text-xs text-zinc-500">
                        {move || t(lang.get(), "contracts.subheading")}
                    </p>
                </div>
                <span class="status-pill">
                    {move || format!("{} listed", history.deploys.with(|d| d.len()))}
                </span>
            </header>

            <Show
                when=move || history.deploys.with(|d| !d.is_empty())
                fallback=|| view! { <ContractsEmpty /> }
            >
                <div class="space-y-2">
                    <For
                        each=move || history.deploys.get()
                        key=|d| d.tx_hash.clone()
                        children=|d: DeployRecord| view! { <DeployRow d /> }
                    />
                </div>
            </Show>
        </section>
    }
}

#[component]
fn DeployRow(d: DeployRecord) -> impl IntoView {
    let tx_link = format!("/tx/{}", d.tx_hash);
    let from_link = format!("/address/{}", d.from);
    let hash_seed = d
        .tx_hash
        .strip_prefix("0x")
        .or_else(|| d.tx_hash.strip_prefix("0X"))
        .unwrap_or(&d.tx_hash)
        .to_string();
    let hash_short = if d.tx_hash.len() >= 14 {
        format!("{}…{}", &d.tx_hash[..10], &d.tx_hash[d.tx_hash.len() - 4..])
    } else {
        d.tx_hash.clone()
    };
    let from_short = if d.from.len() >= 12 {
        format!("{}…{}", &d.from[..6], &d.from[d.from.len() - 4..])
    } else {
        d.from.clone()
    };

    view! {
        <div class="flex items-center justify-between rounded-xl border border-zinc-800/30 bg-zinc-900/40 p-3 text-xs">
            <div class="flex items-center gap-3">
                <div class="identicon-frame h-9 w-9 rounded-lg ring-1 ring-zinc-800/80">
                    <Identicon address_hex=hash_seed size=36 />
                </div>
                <div>
                    <a href=tx_link class="hex font-bold text-zinc-100 hover:text-amber-300">
                        {hash_short}
                    </a>
                    <div class="text-[10px] text-zinc-500">
                        "from "
                        <a href=from_link class="font-mono hover:text-amber-300">
                            {from_short}
                        </a>
                    </div>
                </div>
            </div>
            <div class="text-right">
                <div class="font-mono text-zinc-300">{d.bytecode_len} " bytes"</div>
                <div class="text-[10px] text-zinc-500">{format_when(d.submitted_ms)}</div>
            </div>
        </div>
    }
}

#[component]
fn ContractsEmpty() -> impl IntoView {
    let lang = use_lang();
    view! {
        <div class="rounded-xl border border-dashed border-zinc-800 bg-zinc-900/30 p-10 text-center">
            <h3 class="text-sm font-semibold text-zinc-300">
                {move || t(lang.get(), "contracts.empty_title")}
            </h3>
            <p class="mx-auto mt-2 max-w-md text-xs text-zinc-500">
                {move || t(lang.get(), "contracts.empty_body")}
            </p>
        </div>
    }
}

/// "5m ago" / "2h ago" / "3d ago" / wall-clock fallback. Coarse on
/// purpose — the explorer isn't a stopwatch.
fn format_when(submitted_ms: u64) -> String {
    if submitted_ms == 0 {
        return "—".into();
    }
    let now = now_ms();
    if now <= submitted_ms {
        return "just now".into();
    }
    let delta_ms = now - submitted_ms;
    let s = delta_ms / 1_000;
    if s < 60 {
        format!("{s}s ago")
    } else if s < 3_600 {
        format!("{}m ago", s / 60)
    } else if s < 86_400 {
        format!("{}h ago", s / 3_600)
    } else {
        format!("{}d ago", s / 86_400)
    }
}

#[cfg(target_arch = "wasm32")]
fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> u64 {
    0
}
