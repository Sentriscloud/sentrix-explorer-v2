//! StatsDashboard — 4-card live network stats with Suspense + error
//! boundary. Etherscan-style data, Linear-style polish.
//!
//! Data shape lives in `ChainStats`. The fetcher is mocked behind a
//! 500 ms delay; flip the TODO blocks to real RPC when ready.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::state::network::{use_network, Network};

// ─────────────────────────────────────────────────────────────────
// Data layer
// ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChainStats {
    pub block_height: u64,
    pub avg_block_time_ms: u32,
    pub active_validators: u32,
    pub total_validators: u32,
    /// Live pending-tx depth from `/sentrix_status_extended.mempool.size`.
    /// We don't surface a cumulative tx total — chain RPC has no cheap
    /// query for that (would need a full-chain scan or external indexer).
    /// Pending-tx is honest and live; the card label reflects that.
    pub mempool_pending: u64,
    pub network: Network,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FetchError {
    Rpc(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rpc(s) => write!(f, "rpc: {s}"),
        }
    }
}

impl std::error::Error for FetchError {}

/// Real chain-stats fetcher (wasm path) with SSR mock fallback.
///
/// ## Wired today
///   - `block_height`      → `eth_blockNumber`
///   - `avg_block_time_ms` → mean delta from latest + (latest - 99)
///     timestamps via two `eth_getBlockByNumber` calls
///   - `active_validators` / `total_validators` → REST
///     `/sentrix_status_extended` (validators.active_count and
///     validators.top.len() respectively — top is capped at 7 by
///     stake-rank, accurate as long as registered ≤ 7)
///   - `mempool_pending` → REST `/sentrix_status_extended.mempool.size`
///
/// All four hits go to `network.rpc_url()`; sequential, no procmacro
/// dep. Cumulative tx total is intentionally omitted — chain has no
/// cheap query and a card showing "Pending Tx" is more honest than a
/// stale "Total Transactions" backed by a full-chain scan cache.
#[cfg(target_arch = "wasm32")]
async fn fetch_chain_stats(network: Network) -> Result<ChainStats, FetchError> {
    use crate::api::evm::{EvmProvider, HttpEvmProvider};

    let provider = HttpEvmProvider::new(network.rpc_url());

    let block_height = provider
        .block_number()
        .await
        .map_err(|e| FetchError::Rpc(format!("block_number: {e:?}")))?;

    let avg_block_time_ms = compute_avg_block_time_ms(&provider, block_height).await;

    // Validator counts + mempool depth come from the chain's
    // operator-dashboard endpoint. Single round-trip covers three
    // fields that EVM JSON-RPC can't express. Failure here doesn't
    // tank the whole fetch — fall back to safe zeros so the height
    // and block-time cards still render.
    let extended = fetch_sentrix_status(network).await.unwrap_or_default();

    Ok(ChainStats {
        block_height,
        avg_block_time_ms,
        active_validators: extended.active_validators,
        total_validators: extended.total_validators,
        mempool_pending: extended.mempool_pending,
        network,
    })
}

/// Subset of fields the dashboard cares about. Backed by gRPC v0.4
/// `GetValidatorSet` + `GetMempool` — chain v2.1.72 shipped 2026-05-05
/// across both mainnet (4-validator simul-start) and testnet (docker),
/// so the read-only RPCs are now real on both networks. Replaces the
/// previous REST `/sentrix_status_extended` bridge.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct SentrixStatusSubset {
    active_validators: u32,
    total_validators: u32,
    mempool_pending: u64,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_sentrix_status(_network: Network) -> Result<SentrixStatusSubset, FetchError> {
    use crate::grpc::client::SentrixGrpcClient;

    let mut client = SentrixGrpcClient::new(crate::GRPC_ENDPOINT);
    let validators = client
        .get_validator_set()
        .await
        .map_err(|s| FetchError::Rpc(format!("validator_set: {}", s.message())))?;
    let mempool = client
        .get_mempool(0)
        .await
        .map_err(|s| FetchError::Rpc(format!("mempool: {}", s.message())))?;

    Ok(SentrixStatusSubset {
        active_validators: validators.active_count,
        total_validators: validators.total_count,
        mempool_pending: u64::from(mempool.size),
    })
}

/// Two-point average — latest + (latest - 99) timestamps, divide by
/// the gap. Defaults to 1200 ms (chain target) on any RPC failure
/// or when there aren't enough blocks yet.
#[cfg(target_arch = "wasm32")]
async fn compute_avg_block_time_ms<P: crate::api::evm::EvmProvider>(provider: &P, tip: u64) -> u32 {
    const FALLBACK_MS: u32 = 1_200;
    if tip < 100 {
        return FALLBACK_MS;
    }
    let older = tip - 99;
    let (Ok(t_new), Ok(t_old)) = (
        provider.get_block_by_number(tip).await,
        provider.get_block_by_number(older).await,
    ) else {
        return FALLBACK_MS;
    };
    if t_new.timestamp <= t_old.timestamp {
        return FALLBACK_MS;
    }
    // (Δseconds * 1000) / 99 → millis per block, mean over the window.
    let span_ms = (t_new.timestamp - t_old.timestamp).saturating_mul(1_000);
    u32::try_from(span_ms / 99).unwrap_or(FALLBACK_MS)
}

// ─────────────────────────────────────────────────────────────────
// Component
// ─────────────────────────────────────────────────────────────────

#[component]
pub fn StatsDashboard() -> impl IntoView {
    let network = use_network();

    // We don't use `<Suspense>` here on purpose. Earlier attempts wrapped
    // a `LocalResource` fetcher in Suspense — that registered a streaming
    // chunk in the SSR HTML which never resolved on the server (LocalResource
    // is wasm-only by design). The client hydrator then walked the DOM
    // expecting streamed content, didn't find it, and panicked in
    // `tachys::hydration::failed_to_cast_element` deep in the framework.
    //
    // Plain `RwSignal<State>` + `spawn_local` on the wasm path keeps SSR
    // emitting only the skeleton (zero streaming chunks) and the client
    // populates the real data after hydrate via signal update. Same UX,
    // hydration-clean.
    let state: RwSignal<StatsState> = RwSignal::new(StatsState::Loading);

    // 1s polling matches the chain's block cadence — LATEST BLOCK
    // ticks every block instead of going stale for 5s windows.
    // `/sentrix_status_extended` is a single in-memory snapshot read
    // server-side, cheap enough to hammer at 1Hz from every explorer
    // tab. If load shows up later, switch the high-cadence path to
    // a `StreamEvents([BlockFinalized])` push and downsample REST to
    // 5s for the validator/mempool fields.
    #[cfg(target_arch = "wasm32")]
    {
        let net = network.get_untracked();
        leptos::task::spawn_local(async move {
            loop {
                let next = match fetch_chain_stats(net).await {
                    Ok(s) => StatsState::Ready(s),
                    Err(e) => StatsState::Error(e.to_string()),
                };
                state.set(next);
                crate::util::sleep_ms(1_000).await;
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = network;

    view! {
        <section class="space-y-4" aria-label="Network statistics">
            <NetworkBadge />
            {move || match state.get() {
                StatsState::Loading => view! { <SkeletonGrid /> }.into_any(),
                StatsState::Ready(s) => view! { <StatsGrid stats=s /> }.into_any(),
                StatsState::Error(msg) => view! {
                    <ErrorState
                        message=msg
                        on_retry=move || state.set(StatsState::Loading)
                    />
                }.into_any(),
            }}
            <SupplyBar />
        </section>
    }
}

/// Compact minted/cap progress card. Lives directly under the 4-card
/// stats grid (used to be the StatsPanel's job; that whole panel got
/// dropped because most of its cards were redundant with the hero +
/// the rest had stale "0.00s" data).
#[component]
fn SupplyBar() -> impl IntoView {
    const TOTAL_SUPPLY_SRX: u64 = 315_000_000;
    let minted_srx: RwSignal<u64> = RwSignal::new(0);

    #[cfg(target_arch = "wasm32")]
    {
        use crate::grpc::client::SentrixGrpcClient;
        leptos::task::spawn_local(async move {
            // gRPC `GetSupply` shipped in chain v2.1.72 across both networks
            // 2026-05-05. 5 s poll matches the rest of the panel's slow
            // signals; the headline 1 s poll lives on StatsDashboard above.
            let mut client = SentrixGrpcClient::new(crate::GRPC_ENDPOINT);
            loop {
                if let Ok(supply) = client.get_supply().await {
                    minted_srx.set(supply.minted_sentri / 100_000_000);
                }
                crate::util::sleep_ms(5_000).await;
            }
        });
    }

    let network = use_network();
    view! {
        <div class="rounded-xl border border-zinc-800/60 bg-zinc-900/40 p-4">
            <div class="flex items-center justify-between">
                <span class="eyebrow text-zinc-500">"Minted · Cap"</span>
                {move || {
                    let m = minted_srx.get();
                    let pct = (m * 100) / TOTAL_SUPPLY_SRX.max(1);
                    view! {
                        <span class="font-mono text-xs tabular-nums text-zinc-300">
                            {format_int(m)} " / " {format_int(TOTAL_SUPPLY_SRX)} " SRX · "
                            {pct.to_string()} "%"
                        </span>
                    }
                }}
            </div>
            <div class="mt-3 h-1.5 overflow-hidden rounded-full bg-zinc-800/60">
                <div
                    class=move || format!("h-full rounded-full transition-all duration-700 {}", network.get().accent_bg())
                    style=move || {
                        let m = minted_srx.get();
                        let pct = (m * 100) / TOTAL_SUPPLY_SRX.max(1);
                        format!("width: {pct}%;")
                    }
                />
            </div>
        </div>
    }
}

// Ready/Error are only constructed on the wasm path (SSR sticks at Loading
// forever — that's the desired skeleton-only render). Allow dead-code on the
// SSR analyser pass.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
enum StatsState {
    Loading,
    Ready(ChainStats),
    Error(String),
}

/// Network switcher — uses native `<details>` so it works pre-hydration.
/// Earlier impl was a Leptos `<button>` + `RwSignal<bool>` toggle, but
/// that needs WASM live to wire the click handler — the user reported
/// the dropdown felt unresponsive on testnet (1-2 s WASM hydrate
/// delay). `<details>/<summary>` is browser-native: zero JS to open,
/// links inside navigate cross-subdomain regardless of hydration
/// state. Same UX, no race against WASM.
#[component]
fn NetworkBadge() -> impl IntoView {
    let network = use_network();

    let summary_class = move || {
        let base = "inline-flex cursor-pointer items-center gap-2 rounded-full border px-3 py-1 text-[11px] font-medium tracking-wide list-none transition-colors";
        format!("{base} {}", network.get().accent_pill())
    };
    let dot_class = move || format!("h-1.5 w-1.5 rounded-full {}", network.get().accent_bg());

    view! {
        <details class="group relative inline-block">
            <summary class=summary_class>
                <span class=dot_class></span>
                <span>{move || network.get().label()}</span>
                <span class="font-mono text-[10px] text-zinc-500">
                    "chain · " {move || network.get().chain_id().to_string()}
                </span>
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="h-3 w-3 text-zinc-500 transition-transform group-open:rotate-180"
                >
                    <path d="M6 9l6 6 6-6" />
                </svg>
            </summary>
            <div class="absolute left-0 top-full z-20 mt-1 w-56 overflow-hidden rounded-md border border-zinc-800 bg-zinc-950 shadow-lg">
                <NetworkOption target=Network::Mainnet current=network />
                <NetworkOption target=Network::Testnet current=network />
            </div>
        </details>
    }
}

#[component]
fn NetworkOption(target: Network, current: RwSignal<Network>) -> impl IntoView {
    view! {
        <a
            href=target.explorer_url()
            class="flex items-center justify-between gap-3 px-3 py-2 text-xs transition-colors hover:bg-zinc-900"
        >
            <span class="flex items-center gap-2">
                <span class="w-3 inline-flex justify-center text-emerald-500">
                    {move || if current.get() == target { "✓" } else { "" }}
                </span>
                <span class=format!("inline-flex h-1.5 w-1.5 rounded-full {}", target.accent_bg())></span>
                <span class="font-medium text-zinc-100">{target.label()}</span>
            </span>
            <span class="font-mono text-[10px] tabular-nums text-zinc-500">
                "chain · " {target.chain_id().to_string()}
            </span>
        </a>
    }
}

#[component]
fn StatsGrid(stats: ChainStats) -> impl IntoView {
    let block_time = format!("{:.1}s", f64::from(stats.avg_block_time_ms) / 1000.0);
    let validators = format!("{} / {}", stats.active_validators, stats.total_validators);
    let _ = stats.network;
    let mempool_pending = stats.mempool_pending;
    let height_fallback = stats.block_height;

    view! {
        <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
            <HeroBlockCard height_fallback />

            // Three compact companion cards alongside the hero. Auto rows
            // so each card sizes to its content instead of stretching to
            // 1/3 of the hero height.
            <div class="flex flex-col gap-3 md:col-span-1">
                <StatCard
                    label="Avg Block Time"
                    value=block_time
                    accent=false
                    icon=Icon::Clock
                />
                <StatCard
                    label="Active Validators"
                    value=validators
                    accent=false
                    icon=Icon::Validators
                />
                <StatCard
                    label="Pending Tx"
                    value=format_int(mempool_pending)
                    accent=false
                    icon=Icon::Transactions
                />
            </div>
        </div>
    }
}

/// Hero block card — reads `BlockFeedState` from context so it carries
/// hash + proposer + tx_count + timestamp alongside the height. Falls
/// back to the REST height (`stats.block_height`) for the brief window
/// before the gRPC stream's first block lands.
///
/// Identicon stays on the per-row tiles only — the hero stays clean
/// (#height + truncated hash + metadata row), no leading visual.
#[component]
fn HeroBlockCard(height_fallback: u64) -> impl IntoView {
    use crate::labels::{label_for, Label};
    use crate::state::feed::{BlockFeedState, BlockRow};

    let feed = use_context::<BlockFeedState>().expect("BlockFeedState context");
    let network = use_network();

    let latest = Memo::new(move |_| feed.blocks.with(|b| b.first().cloned()));

    let accent_text_class = move || network.get().accent_text();
    let pulse_dot_class = move || {
        format!(
            "relative inline-flex h-2 w-2 rounded-full {}",
            network.get().accent_bg()
        )
    };
    let pulse_ping_class = move || {
        format!(
            "absolute inline-flex h-full w-full animate-ping rounded-full {} opacity-70",
            network.get().accent_bg()
        )
    };

    view! {
        <article
            class="corner-lines relative md:col-span-2 rounded-xl border border-zinc-800/60 bg-zinc-900/40 p-6 transition-colors hover:border-emerald-700/40"
            aria-label="Latest Block"
        >
            <header class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <span class="relative flex h-2 w-2">
                        <span class=pulse_ping_class></span>
                        <span class=pulse_dot_class></span>
                    </span>
                    <span class="eyebrow text-zinc-500">"Latest Block"</span>
                </div>
                <span class=accent_text_class>
                    <IconSvg icon=Icon::Block />
                </span>
            </header>

            {move || match latest.get() {
                Some(row) => {
                    let row_for_render: BlockRow = row;
                    let hash_short = if row_for_render.hash_hex.len() >= 8 {
                        format!(
                            "0x{}…{}",
                            &row_for_render.hash_hex[..4],
                            &row_for_render.hash_hex[row_for_render.hash_hex.len() - 4..]
                        )
                    } else {
                        format!("0x{}", row_for_render.hash_hex)
                    };
                    let proposer_full = format!("0x{}", row_for_render.proposer_hex);
                    let proposer_label = label_for(&proposer_full, network.get())
                        .map(|l: Label| l.name.to_string())
                        .unwrap_or_else(|| {
                            if row_for_render.proposer_hex.len() >= 6 {
                                format!("0x{}…", &row_for_render.proposer_hex[..6])
                            } else {
                                proposer_full.clone()
                            }
                        });
                    let height_fmt = format_int(row_for_render.height);
                    let tx_count = row_for_render.tx_count;
                    let timestamp = row_for_render.timestamp;
                    let height_class = format!(
                        "mt-4 font-serif text-6xl font-bold tabular-nums tracking-tight {}",
                        network.get().accent_text()
                    );

                    view! {
                        <div class=height_class>
                            "#" {height_fmt}
                        </div>
                        <div class="mt-2 hex break-all text-sm text-zinc-500">{hash_short}</div>
                        <div class="mt-4 flex flex-wrap items-center gap-2 border-t border-zinc-800/40 pt-3 text-[11px] text-zinc-500">
                            <span class="font-mono tabular-nums text-zinc-300">
                                {tx_count} " txs"
                            </span>
                            <span class="text-zinc-700">"·"</span>
                            <span class="font-mono">{proposer_label}</span>
                            <span class="text-zinc-700">"·"</span>
                            <span class="font-mono tabular-nums">
                                {move || format_relative_short(timestamp)}
                            </span>
                        </div>
                    }
                    .into_any()
                }
                None => {
                    let height_class = format!(
                        "mt-4 font-serif text-6xl font-bold tabular-nums tracking-tight {}",
                        network.get().accent_text()
                    );
                    view! {
                        <div class=height_class>
                            "#" {format_int(height_fallback)}
                        </div>
                        <div class="mt-3 text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                            "Connecting to live feed…"
                        </div>
                    }
                    .into_any()
                }
            }}
        </article>
    }
}

#[cfg(target_arch = "wasm32")]
fn format_relative_short(ts: u64) -> String {
    let now = (js_sys::Date::now() / 1000.0) as u64;
    relative_short(now, ts)
}

#[cfg(not(target_arch = "wasm32"))]
fn format_relative_short(ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(ts);
    relative_short(now, ts)
}

fn relative_short(now: u64, ts: u64) -> String {
    let delta = now.saturating_sub(ts);
    if delta < 5 {
        "just now".into()
    } else if delta < 60 {
        format!("{delta}s ago")
    } else if delta < 3600 {
        format!("{} min ago", delta / 60)
    } else if delta < 86400 {
        format!("{} hr ago", delta / 3600)
    } else {
        format!("{} d ago", delta / 86400)
    }
}

#[derive(Clone, Copy)]
enum Icon {
    Block,
    Clock,
    Validators,
    Transactions,
}

#[component]
fn StatCard(label: &'static str, value: String, accent: bool, icon: Icon) -> impl IntoView {
    let value_class = if accent {
        "mt-2 font-mono text-2xl font-bold tabular-nums text-emerald-500"
    } else {
        "mt-2 font-mono text-2xl font-bold tabular-nums text-zinc-100"
    };

    view! {
        <article
            class="group corner-lines relative rounded-xl border border-zinc-800/60 bg-zinc-900/40 px-4 py-3.5 transition-colors hover:border-emerald-700/40"
            aria-label=label
        >
            <header class="flex items-center justify-between">
                <span class="eyebrow text-zinc-500">{label}</span>
                <IconSvg icon />
            </header>
            <div class=value_class>{value}</div>
        </article>
    }
}

#[component]
fn IconSvg(icon: Icon) -> impl IntoView {
    // Lucide-shape paths inlined so we don't pull a JSX-style icon
    // crate. Keep `stroke="currentColor"` so Solar Mode + hover
    // tints work via the parent's text color.
    let path = match icon {
        Icon::Block => {
            "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16zM3.27 6.96L12 12.01l8.73-5.05M12 22.08V12"
        }
        Icon::Clock => "M12 6v6l4 2M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0z",
        Icon::Validators => {
            "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"
        }
        Icon::Transactions => "M16 3l4 4-4 4M20 7H4M8 21l-4-4 4-4M4 17h16",
    };
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            class="h-4 w-4 text-zinc-600 transition-colors group-hover:text-emerald-500"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d=path />
        </svg>
    }
}

#[component]
fn SkeletonGrid() -> impl IntoView {
    view! {
        <div
            class="grid grid-cols-1 gap-3 md:grid-cols-3"
            role="status"
            aria-label="Loading statistics"
        >
            <div class="md:col-span-2 rounded-xl border border-zinc-800/60 bg-zinc-900/40 p-6">
                <div class="skeleton-shimmer h-3 w-24 rounded bg-zinc-800" />
                <div class="skeleton-shimmer mt-4 h-12 w-56 rounded bg-zinc-800" />
                <div class="skeleton-shimmer mt-3 h-3 w-32 rounded bg-zinc-800" />
            </div>
            <div class="grid grid-cols-1 gap-3 md:grid-rows-3">
                {(0..3).map(|_| view! { <SkeletonCard /> }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn SkeletonCard() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <div class="skeleton-shimmer h-3 w-24 rounded bg-slate-800" />
            <div class="skeleton-shimmer mt-4 h-7 w-32 rounded bg-slate-800" />
            <div class="skeleton-shimmer mt-3 h-3 w-16 rounded bg-slate-800" />
        </div>
    }
}

#[component]
fn ErrorState<F>(message: String, on_retry: F) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <div
            role="alert"
            class="rounded-xl border border-rose-500/30 bg-rose-500/5 p-5"
        >
            <div class="text-sm font-semibold text-rose-200">
                "Failed to load network statistics"
            </div>
            <div class="mt-1 font-mono text-xs text-rose-300/80">{message}</div>
            <button
                type="button"
                on:click=move |_| on_retry()
                class="mt-3 rounded-md border border-rose-500/40 bg-rose-500/10 px-3 py-1 text-xs font-medium text-rose-200 transition hover:bg-rose-500/20"
            >
                "Retry"
            </button>
        </div>
    }
}

/// Thousands-separator formatter — std doesn't ship one and pulling
/// `num-format` for 8 lines isn't worth the WASM bytes.
fn format_int(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}
