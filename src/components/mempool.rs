//! MempoolWatcher — pending transactions panel.
//!
//! Pure consumer of `MempoolState` from context — no gRPC IO here.

use leptos::prelude::*;

use crate::components::identicon::Identicon;
use crate::i18n::{t, use_lang};
use crate::state::mempool::{MempoolState, PendingTxRow};

#[component]
pub fn MempoolWatcher() -> impl IntoView {
    let m = use_context::<MempoolState>().expect("MempoolState context");
    let lang = use_lang();

    view! {
        <section class="glass-card rounded-2xl p-6">
            <header class="mb-4 flex items-center justify-between">
                <div>
                    <div class="eyebrow text-zinc-500">"Mempool · Pending"</div>
                    <h2 class="mt-1 font-serif text-2xl font-bold tracking-tight text-zinc-100">
                        {move || t(lang.get(), "mempool.pending")}
                    </h2>
                </div>
                <span class="status-pill">{move || m.status.get()}</span>
            </header>

            <div class="space-y-2">
                <Show
                    when=move || m.pending.with(|p| !p.is_empty())
                    fallback=|| view! { <PendingEmpty /> }
                >
                    <For
                        each=move || m.pending.get()
                        key=|row| row.txid_hex.clone()
                        children=|row: PendingTxRow| {
                            view! { <PendingTile row /> }
                        }
                    />
                </Show>
            </div>
        </section>
    }
}

#[component]
fn PendingTile(row: PendingTxRow) -> impl IntoView {
    let from_short = format!("0x{}", &row.from_hex[..6]);
    let to_short = format!("0x{}", &row.to_hex[..6]);
    let txid_short = format!("{}…", &row.txid_hex[..10]);
    let amount = format_sentri(row.amount_sentri);
    let fee = format_sentri(row.fee_sentri);
    let kind = tx_type_label(row.tx_type);
    let identicon_seed = row.from_hex.clone();

    view! {
        <div class="flex items-center justify-between rounded-xl border border-zinc-800/30 bg-zinc-900/40 p-3 transition-all hover:border-zinc-700">
            <div class="flex items-center gap-3">
                <div class="identicon-frame h-8 w-8 rounded-lg ring-1 ring-zinc-800/80">
                    <Identicon address_hex=identicon_seed size=32 />
                </div>
                <div>
                    <div class="text-xs text-zinc-400">
                        <span class="font-mono">{from_short}</span>
                        <span class="px-1 text-zinc-600">"→"</span>
                        <span class="font-mono">{to_short}</span>
                    </div>
                    <div class="hex text-[10px] text-zinc-600">{txid_short}</div>
                </div>
            </div>
            <div class="text-right">
                <div class="font-mono text-sm font-bold text-amber-300">{amount} " SRX"</div>
                <div class="text-[10px] text-zinc-500">{kind} " · fee " {fee}</div>
            </div>
        </div>
    }
}

#[component]
fn PendingEmpty() -> impl IntoView {
    // No skeleton spinner here. The mempool legitimately stays empty
    // when chain traffic is light, and a permanent shimmer reads as
    // "broken" instead of "idle". Editorial copy beats fake activity.
    view! {
        <div class="flex flex-col items-center justify-center gap-3 py-10 text-center">
            <div class="font-serif text-4xl font-bold tabular-nums text-zinc-700">"0"</div>
            <div class="space-y-1">
                <div class="eyebrow text-zinc-500">"Mempool · Idle"</div>
                <p class="max-w-xs text-xs text-zinc-500">
                    "Tidak ada transaksi yang menunggu. Saat traffic naik, transaksi muncul di sini sebelum di-finalize."
                </p>
            </div>
        </div>
    }
}

/// Sentri (10⁻⁸ SRX) → human SRX with 4 decimal places.
fn format_sentri(sentri: u64) -> String {
    let whole = sentri / 100_000_000;
    let frac = (sentri % 100_000_000) / 10_000;
    format!("{whole}.{frac:04}")
}

fn tx_type_label(t: u32) -> &'static str {
    match t {
        0 => "transfer",
        1 => "contract",
        2 => "staking-op",
        _ => "unknown",
    }
}
