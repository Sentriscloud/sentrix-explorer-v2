//! `/address/:addr` — address detail screen.
//!
//! Routes to one of three bodies based on `classify_address`:
//!
//! - `Evm(0x…)` — fetch balance + code via the EVM RPC; render
//!   "EOA" or "Contract" with the balance shown as ETH-style 18-dec.
//!   (Sentrix's native unit is sentri = 1e-8 SRX, but the EVM side
//!   exposes the standard 18-decimal wei layer for tooling parity.)
//! - `Hash32` — surface as a tx/block hash; offer cross-link.
//! - other — show "unknown" affordance.
//!
//! Uses an `Either` chain so each branch keeps its native render
//! logic without boxing.

use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::api::{classify_address, AddressKind, NativeProvider, SentrixNativeProvider};
use crate::components::contract_panels::{ContractReadPanel, LogsPanel};
use crate::components::copy_cli::CopyCli;
use crate::components::identicon::Identicon;
use crate::i18n::{t, use_lang};

#[component]
pub fn AddressDetailScreen() -> impl IntoView {
    let params = use_params_map();
    let raw = params.read().get("addr").unwrap_or_default();
    let kind = classify_address(&raw);

    let title = format!("Sentrix · {raw}");

    view! {
        <Title text=title />

        <section class="glass-card space-y-4 rounded-2xl p-6">
            <header class="flex items-center gap-4">
                <div class="identicon-frame h-12 w-12 rounded-lg ring-1 ring-zinc-800/80">
                    <Identicon address_hex=raw.clone() size=48 />
                </div>
                <div>
                    <div class="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                        {kind_label(&kind)}
                    </div>
                    <h1 class="hex break-all text-lg font-bold text-zinc-100">{raw.clone()}</h1>
                </div>
            </header>

            {match kind.clone() {
                AddressKind::Evm(addr) => {
                    EitherOf3::A(view! { <EvmBody addr /> })
                }
                AddressKind::Hash32(_) => EitherOf3::B(view! { <HashBody raw=raw.clone() /> }),
                _ => EitherOf3::C(view! { <UnknownBody /> }),
            }}

            <footer class="border-t border-zinc-800/40 pt-4">
                <div class="mb-2 text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                    "Sentrix CLI"
                </div>
                <CopyCli command=cli_for(&kind, &raw) />
            </footer>
        </section>
    }
}

fn kind_label(k: &AddressKind) -> &'static str {
    match k {
        AddressKind::Evm(_) => "EVM Address",
        AddressKind::Hash32(_) => "Hash",
        AddressKind::BlockHeight(_) => "Block",
        AddressKind::Unknown => "Unknown",
    }
}

fn cli_for(k: &AddressKind, raw: &str) -> String {
    match k {
        AddressKind::Evm(addr) => format!("srx-cli evm balance {addr}"),
        AddressKind::Hash32(_) => format!("srx-cli get tx --hash {raw}"),
        AddressKind::BlockHeight(h) => format!("srx-cli get block --height {h}"),
        AddressKind::Unknown => format!("# unrecognized: {raw}"),
    }
}

#[component]
fn EvmBody(addr: String) -> impl IntoView {
    let addr_for_balance = addr.clone();
    let addr_for_code = addr.clone();
    let addr_for_native = addr.clone();
    let addr_for_panels = addr.clone();

    let balance = LocalResource::new(move || {
        let a = addr_for_balance.clone();
        async move {
            #[cfg(target_arch = "wasm32")]
            {
                use crate::api::{EvmProvider, HttpEvmProvider};
                let p = HttpEvmProvider::default_for_network();
                p.get_balance_wei(&a).await.ok()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = a;
                None::<u128>
            }
        }
    });

    let code = LocalResource::new(move || {
        let a = addr_for_code.clone();
        async move {
            #[cfg(target_arch = "wasm32")]
            {
                use crate::api::{EvmProvider, HttpEvmProvider};
                let p = HttpEvmProvider::default_for_network();
                p.get_code(&a).await.ok()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = a;
                None::<Vec<u8>>
            }
        }
    });

    // Native-side balance via gRPC. Sentrix addresses are 20-byte
    // even on the native plane, so the same hex address routes
    // through `GetBalance`. Returns the canonical sentri amount —
    // useful as a parity check against the EVM wei view.
    let native_balance = LocalResource::new(move || {
        let a = addr_for_native.clone();
        async move {
            let bytes = parse_h160(&a)?;
            let p = SentrixNativeProvider::default_for_network();
            p.get_balance(bytes)
                .await
                .ok()
                .and_then(|acc| acc.balance.map(|b| (b.sentri, acc.nonce)))
        }
    });

    view! {
        <dl class="space-y-3 text-sm">
            <Row label_key="detail.evm_balance">
                <Suspense fallback=|| view! { <span class="hex">"…"</span> }>
                    {move || Suspend::new(async move {
                        let bal = balance.await;
                        match bal {
                            Some(wei) => format_wei(wei),
                            None => "—".to_string(),
                        }
                    })}
                </Suspense>
            </Row>
            <Row label_key="detail.native_balance">
                <Suspense fallback=|| view! { <span class="hex">"…"</span> }>
                    {move || Suspend::new(async move {
                        match native_balance.await {
                            Some((sentri, _nonce)) => format_sentri(sentri),
                            None => "—".to_string(),
                        }
                    })}
                </Suspense>
            </Row>
            <Row label_key="detail.nonce">
                <Suspense fallback=|| view! { <span class="hex">"…"</span> }>
                    {move || Suspend::new(async move {
                        match native_balance.await {
                            Some((_, nonce)) => nonce.to_string(),
                            None => "—".to_string(),
                        }
                    })}
                </Suspense>
            </Row>
            <Row label_key="detail.account_type">
                <Suspense fallback=|| view! { <span class="hex">"…"</span> }>
                    {move || Suspend::new(async move {
                        match code.await {
                            Some(c) if !c.is_empty() => {
                                format!("Contract · {} bytes", c.len())
                            }
                            Some(_) => "EOA".to_string(),
                            None => "—".to_string(),
                        }
                    })}
                </Suspense>
            </Row>
        </dl>

        // Contract-only panels — render unconditionally for any
        // 0x-shaped address. Logs may return empty for an EOA; the
        // Read panel falls through to RPC errors on non-contracts,
        // which is the right signal anyway.
        <div class="mt-6 space-y-4">
            <LogsPanel addr=addr_for_panels.clone() />
            <ContractReadPanel addr=addr_for_panels />
        </div>
    }
}

/// Parse an `0x`-prefixed 40-char hex string into a 20-byte array.
/// `None` on length mismatch or non-hex characters.
fn parse_h160(s: &str) -> Option<[u8; 20]> {
    let cleaned = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    if cleaned.len() != 40 {
        return None;
    }
    let bytes = hex::decode(cleaned).ok()?;
    let mut out = [0u8; 20];
    out.copy_from_slice(&bytes);
    Some(out)
}

/// Sentri (10⁻⁸ SRX) → human SRX with 4 decimal places.
fn format_sentri(sentri: u64) -> String {
    let whole = sentri / 100_000_000;
    let frac = (sentri % 100_000_000) / 10_000;
    format!("{whole}.{frac:04} SRX")
}

#[component]
fn HashBody(raw: String) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-dashed border-zinc-800 bg-zinc-900/30 p-6 text-sm text-zinc-400">
            "This looks like a 32-byte hash. Try the "
            <a
                href=format!("/tx/{raw}")
                class="text-amber-300 hover:underline"
            >
                "transaction view"
            </a>
            " — the page falls back to a block lookup if the hash isn't in the tx index."
        </div>
    }
}

#[component]
fn UnknownBody() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-dashed border-zinc-800 bg-zinc-900/30 p-6 text-sm text-zinc-500">
            "Unrecognised address format. Sentrix supports 20-byte EVM addresses (40 hex chars after `0x`) and 32-byte hashes."
        </div>
    }
}

#[component]
fn Row(label_key: &'static str, children: Children) -> impl IntoView {
    let lang = use_lang();
    view! {
        <div class="flex items-center justify-between border-b border-zinc-800/40 pb-2 last:border-b-0 last:pb-0">
            <dt class="text-xs uppercase tracking-wider text-zinc-500">
                {move || t(lang.get(), label_key)}
            </dt>
            <dd class="font-mono text-sm text-zinc-200">{children()}</dd>
        </div>
    }
}

/// 18-decimal wei → SRX with up to 6 fractional digits, trimmed.
fn format_wei(wei: u128) -> String {
    if wei == 0 {
        return "0 SRX".into();
    }
    let whole = wei / 1_000_000_000_000_000_000u128;
    let frac = (wei % 1_000_000_000_000_000_000u128) / 1_000_000_000_000u128;
    if frac == 0 {
        format!("{whole} SRX")
    } else {
        format!("{whole}.{frac:06} SRX")
    }
}
