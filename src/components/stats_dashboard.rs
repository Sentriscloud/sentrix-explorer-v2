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

/// Subset of `/sentrix_status_extended` the dashboard cares about.
/// `Default` returns zero-valued fields so a partial-failure path can
/// keep rendering the JSON-RPC-backed cards without short-circuiting.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct SentrixStatusSubset {
    active_validators: u32,
    total_validators: u32,
    mempool_pending: u64,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_sentrix_status(network: Network) -> Result<SentrixStatusSubset, FetchError> {
    use gloo_net::http::Request;
    use serde_json::Value;

    let url = format!("{}/sentrix_status_extended", network.rpc_url());
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| FetchError::Rpc(format!("status: {e}")))?;
    if !resp.ok() {
        return Err(FetchError::Rpc(format!("status http {}", resp.status())));
    }
    let body: Value = resp
        .json()
        .await
        .map_err(|e| FetchError::Rpc(format!("status decode: {e}")))?;

    // Pull fields defensively — every one of these has a chain-side
    // type guarantee, but a future field rename shouldn't crash the
    // dashboard. Missing field == 0; UI shows "0 / 0" which is
    // recognisable as "endpoint changed shape" without a panic.
    let active_validators = body
        .pointer("/validators/active_count")
        .and_then(Value::as_u64)
        .map(|n| u32::try_from(n).unwrap_or(u32::MAX))
        .unwrap_or(0);

    // `top` is capped at 7 by stake-rank server-side; for mainnet's
    // current 4-validator set this equals the registered count. Once
    // external validators push registered > 7, swap to a dedicated
    // count field on the endpoint or a /staking/validators call.
    let total_validators = body
        .pointer("/validators/top")
        .and_then(Value::as_array)
        .map(|a| u32::try_from(a.len()).unwrap_or(u32::MAX))
        .unwrap_or(0);

    let mempool_pending = body
        .pointer("/mempool/size")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    Ok(SentrixStatusSubset {
        active_validators,
        total_validators,
        mempool_pending,
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
        </section>
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

#[component]
fn NetworkBadge() -> impl IntoView {
    let network = use_network();
    view! {
        <div class="flex items-center gap-2" role="status" aria-live="polite">
            <span class=move || {
                let base = "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium";
                match network.get() {
                    Network::Mainnet => format!("{base} border-emerald-500/30 bg-emerald-500/10 text-emerald-300"),
                    Network::Testnet => format!("{base} border-amber-500/30 bg-amber-500/10 text-amber-300"),
                }
            }>
                <span class=move || match network.get() {
                    Network::Mainnet => "h-1.5 w-1.5 rounded-full bg-emerald-500",
                    Network::Testnet => "h-1.5 w-1.5 rounded-full bg-amber-500",
                } />
                {move || network.get().label()}
            </span>
            <span class="hex text-[10px] text-slate-500">
                "chain id · " {move || network.get().chain_id().to_string()}
            </span>
        </div>
    }
}

#[component]
fn StatsGrid(stats: ChainStats) -> impl IntoView {
    let block_time = format!("{:.1}s", f64::from(stats.avg_block_time_ms) / 1000.0);
    let validators = format!("{} / {}", stats.active_validators, stats.total_validators);
    let _ = stats.network;

    view! {
        <div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-4">
            <StatCard
                label="Latest Block"
                value=format_int(stats.block_height)
                accent=true
                icon=Icon::Block
            />
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
                value=format_int(stats.mempool_pending)
                accent=false
                icon=Icon::Transactions
            />
        </div>
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
fn StatCard(
    label: &'static str,
    value: String,
    accent: bool,
    icon: Icon,
) -> impl IntoView {
    let value_class = if accent {
        "mt-2.5 font-mono text-3xl font-bold tabular-nums text-sentrix-gold"
    } else {
        "mt-2.5 font-mono text-3xl font-bold tabular-nums text-zinc-100"
    };

    view! {
        <article
            class="group corner-lines relative rounded-xl border border-zinc-800/60 bg-zinc-900/40 p-5 transition-colors hover:border-sentrix-bronze/40"
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
            class="h-4 w-4 text-zinc-600 transition-colors group-hover:text-sentrix-gold"
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
            class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4"
            role="status"
            aria-label="Loading statistics"
        >
            {(0..4).map(|_| view! { <SkeletonCard /> }).collect_view()}
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
