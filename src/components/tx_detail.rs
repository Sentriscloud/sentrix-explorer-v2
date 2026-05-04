//! TxDetail — interactive transaction decoder.
//!
//! Two display modes:
//!   - "Human" — labelled fields, decoded `tx_type`, formatted SRX
//!   - "Raw"   — pretty-printed JSON of the underlying decoded data
//!
//! No `syntect`. The dependency is heavyweight in WASM (grammars +
//! theme bytes ≈ 1-2 MB). For the level of highlight we need (just
//! "code aesthetic"), tailwind classes on a `<pre>` deliver the same
//! visual result with zero added bytes.

use leptos::prelude::*;

use crate::components::identicon::Identicon;
use crate::state::mempool::PendingTxRow;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Human,
    Raw,
}

#[component]
pub fn TxDetail(row: PendingTxRow) -> impl IntoView {
    let (mode, set_mode) = signal(Mode::Human);
    let row_for_view = row.clone();
    let row_for_raw = row.clone();

    let toggle = move |target: Mode| {
        let active = move || mode.get() == target;
        let label = match target {
            Mode::Human => "Human",
            Mode::Raw => "Raw",
        };
        view! {
            <button
                type="button"
                on:click=move |_| set_mode.set(target)
                class=move || {
                    let base = "rounded-md px-3 py-1 text-xs font-medium transition";
                    if active() {
                        format!("{base} bg-amber-500/15 text-amber-300 ring-1 ring-amber-500/30")
                    } else {
                        format!("{base} text-zinc-400 hover:text-zinc-200")
                    }
                }
            >
                {label}
            </button>
        }
    };

    view! {
        <section class="glass-card rounded-2xl p-6">
            <header class="mb-4 flex items-center justify-between">
                <h2 class="text-xl font-bold italic tracking-tighter text-zinc-100">
                    "TRANSACTION"
                </h2>
                <div class="flex items-center gap-1 rounded-lg border border-zinc-800 bg-zinc-900/60 p-1">
                    {toggle(Mode::Human)}
                    {toggle(Mode::Raw)}
                </div>
            </header>

            <Show
                when=move || mode.get() == Mode::Human
                fallback=move || view! { <RawJson row=row_for_raw.clone() /> }
            >
                <HumanView row=row_for_view.clone() />
            </Show>
        </section>
    }
}

#[component]
fn HumanView(row: PendingTxRow) -> impl IntoView {
    let from = format!("0x{}", row.from_hex);
    let to = format!("0x{}", row.to_hex);
    let action = decode_action(row.tx_type);
    let identicon_from = row.from_hex.clone();
    let identicon_to = row.to_hex.clone();

    view! {
        <dl class="space-y-3 text-sm">
            <Field label="Action">
                <span class="font-mono text-amber-300">{action}</span>
            </Field>
            <Field label="From">
                <div class="flex items-center gap-2">
                    <div class="h-5 w-5 overflow-hidden rounded">
                        <Identicon address_hex=identicon_from size=20 />
                    </div>
                    <span class="hex text-xs">{from}</span>
                </div>
            </Field>
            <Field label="To">
                <div class="flex items-center gap-2">
                    <div class="h-5 w-5 overflow-hidden rounded">
                        <Identicon address_hex=identicon_to size=20 />
                    </div>
                    <span class="hex text-xs">{to}</span>
                </div>
            </Field>
            <Field label="Amount">
                <span class="font-mono text-zinc-200">
                    {format_sentri(row.amount_sentri)} " SRX"
                </span>
            </Field>
            <Field label="Fee">
                <span class="font-mono text-zinc-400">
                    {format_sentri(row.fee_sentri)} " SRX"
                </span>
            </Field>
        </dl>
    }
}

#[component]
fn Field(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between border-b border-zinc-800/40 pb-2 last:border-b-0 last:pb-0">
            <dt class="text-xs uppercase tracking-wider text-zinc-500">{label}</dt>
            <dd>{children()}</dd>
        </div>
    }
}

#[component]
fn RawJson(row: PendingTxRow) -> impl IntoView {
    let json = serialize_human_readable(&row);
    view! {
        <pre class="hex max-h-80 overflow-auto rounded-lg bg-black/40 p-4 text-[11px] leading-relaxed text-zinc-300">
            {json}
        </pre>
    }
}

/// Human-readable label for the chain's `tx_type`. Values 0/1/2 are
/// the canonical `transfer | contract | staking-op` triple from the
/// proto. Higher values are reserved for future RWA-event encodings;
/// they fall through to a placeholder until those discriminants ship.
fn decode_action(tx_type: u32) -> &'static str {
    match tx_type {
        0 => "Transfer",
        1 => "Contract Call",
        2 => "Staking Operation",
        _ => "Unknown / RWA event (decoder pending)",
    }
}

fn format_sentri(sentri: u64) -> String {
    let whole = sentri / 100_000_000;
    let frac = (sentri % 100_000_000) / 10_000;
    format!("{whole}.{frac:04}")
}

/// Hand-rolled JSON of the visible row. Keeps the WASM bundle slim
/// (no `serde_json` for one struct) and gives us full control over
/// the rendering — hashes stay 0x-prefixed, sentri amounts surface
/// alongside their human SRX value.
fn serialize_human_readable(r: &PendingTxRow) -> String {
    format!(
        "{{\n  \"txid\":   \"0x{}\",\n  \"from\":   \"0x{}\",\n  \"to\":     \"0x{}\",\n  \"amount\": {{ \"sentri\": {}, \"srx\": \"{}\" }},\n  \"fee\":    {{ \"sentri\": {}, \"srx\": \"{}\" }},\n  \"tx_type\": {} ({})\n}}",
        r.txid_hex,
        r.from_hex,
        r.to_hex,
        r.amount_sentri,
        format_sentri(r.amount_sentri),
        r.fee_sentri,
        format_sentri(r.fee_sentri),
        r.tx_type,
        decode_action(r.tx_type),
    )
}
