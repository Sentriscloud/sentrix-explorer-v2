//! Per-contract panels rendered on `/address/:addr` when the
//! account_type is Contract.
//!
//! - `LogsPanel` — `eth_getLogs` over a recent-block window. Renders
//!   topics + data as raw hex (no ABI decode); the Transfer event
//!   signature is highlighted as a hint.
//! - `ContractReadPanel` — wraps an `eth_call` with the address
//!   pre-bound, so users can probe view methods without leaving the
//!   page.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::LogEntry;
use crate::state::feed::BlockFeedState;

const TRANSFER_TOPIC: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum LogsState {
    Idle,
    Loading,
    Loaded(Vec<LogEntry>),
    Err(String),
}

#[component]
pub fn LogsPanel(addr: String) -> impl IntoView {
    let (state, set_state) = signal(LogsState::Idle);
    let feed = use_context::<BlockFeedState>().expect("BlockFeedState context");

    let load = {
        let addr = addr.clone();
        move |_| {
            // Window: last 1000 blocks ending at the current tip. We
            // can't ask "since contract creation" without an indexer;
            // 1000 is a reasonable browse window matching what
            // public RPCs typically allow without a paid plan.
            let tip = feed
                .blocks
                .with(|b| b.first().map(|r| r.height))
                .unwrap_or(0);
            let from = tip.saturating_sub(1_000);
            let addr = addr.clone();
            set_state.set(LogsState::Loading);
            spawn_local(async move {
                #[cfg(target_arch = "wasm32")]
                {
                    use crate::api::{EvmProvider, HttpEvmProvider};
                    let p = HttpEvmProvider::default_for_network();
                    match p.get_logs(&addr, from, tip).await {
                        Ok(logs) => set_state.set(LogsState::Loaded(logs)),
                        Err(e) => set_state.set(LogsState::Err(format!("{e:?}"))),
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = (addr, from, tip);
                    set_state.set(LogsState::Err("ssr".into()));
                }
            });
        }
    };

    view! {
        <section class="space-y-3 rounded-2xl border border-zinc-800/40 bg-zinc-900/40 p-4">
            <div class="flex items-center justify-between">
                <h3 class="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                    "Event logs · last 1000 blocks"
                </h3>
                <button
                    type="button"
                    on:click=load
                    class="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-amber-200 transition hover:bg-amber-500/20"
                >
                    {move || match state.get() {
                        LogsState::Loading => "loading…",
                        _ => "fetch logs",
                    }}
                </button>
            </div>

            {move || match state.get() {
                LogsState::Idle => Some(
                    view! {
                        <p class="text-xs text-zinc-500">
                            "Click 'fetch logs' to query eth_getLogs against this address."
                        </p>
                    }.into_any()
                ),
                LogsState::Loading => Some(
                    view! { <p class="text-xs text-zinc-400">"Querying RPC…"</p> }.into_any()
                ),
                LogsState::Err(e) => Some(
                    view! {
                        <p class="text-xs text-rose-300">{format!("error · {e}")}</p>
                    }.into_any()
                ),
                LogsState::Loaded(logs) if logs.is_empty() => Some(
                    view! {
                        <p class="text-xs text-zinc-500">
                            "No logs in this window."
                        </p>
                    }.into_any()
                ),
                LogsState::Loaded(logs) => Some(
                    view! { <LogsList logs /> }.into_any()
                ),
            }}
        </section>
    }
}

#[component]
fn LogsList(logs: Vec<LogEntry>) -> impl IntoView {
    view! {
        <div class="space-y-2">
            {logs.into_iter().map(|l| view! { <LogRow l /> }).collect_view()}
        </div>
    }
}

#[component]
fn LogRow(l: LogEntry) -> impl IntoView {
    let tx_link = format!("/tx/{}", l.tx_hash);
    let block_link = format!("/block/{}", l.block_number);
    let event_label = l
        .topics
        .first()
        .and_then(|t| label_for_topic(t))
        .unwrap_or("event");
    let data_short = if l.data.len() > 20 {
        format!("{}…", &l.data[..20])
    } else {
        l.data.clone()
    };

    view! {
        <div class="rounded-lg border border-zinc-800/40 bg-zinc-900/30 p-3 text-xs">
            <div class="flex items-center justify-between">
                <span class="font-mono text-amber-300">{event_label}</span>
                <a href=tx_link.clone() class="hex text-[10px] text-zinc-400 hover:text-amber-300">
                    {short_hex(&l.tx_hash)}
                </a>
            </div>
            <div class="mt-1 flex items-center justify-between text-[10px] text-zinc-500">
                <a href=block_link class="font-mono hover:text-amber-300">
                    "block #" {l.block_number}
                </a>
                <span class="font-mono">"index " {l.log_index}</span>
            </div>
            {if !l.topics.is_empty() {
                let topic_lines = l.topics
                    .iter()
                    .skip(1)
                    .enumerate()
                    .map(|(i, t)| view! {
                        <div class="hex truncate text-[10px] text-zinc-400">
                            {format!("topic {}: ", i + 1)}{t.clone()}
                        </div>
                    })
                    .collect_view();
                Some(view! { <div class="mt-2 space-y-0.5">{topic_lines}</div> }.into_any())
            } else { None }}
            {if !l.data.is_empty() && l.data != "0x" {
                Some(view! {
                    <div class="hex mt-1 truncate text-[10px] text-zinc-500">
                        "data: " {data_short}
                    </div>
                }.into_any())
            } else { None }}
        </div>
    }
}

fn label_for_topic(topic: &str) -> Option<&'static str> {
    // Only the canonical Transfer signature is recognised today.
    // Extending here is just match arms; a real ABI registry is out
    // of scope until we ship the canonical-contracts feature.
    if topic.eq_ignore_ascii_case(TRANSFER_TOPIC) {
        Some("Transfer (ERC-20/721)")
    } else {
        None
    }
}

fn short_hex(s: &str) -> String {
    if s.len() >= 14 {
        format!("{}…{}", &s[..10], &s[s.len() - 4..])
    } else {
        s.to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Contract Read panel
// ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum ReadState {
    Idle,
    Calling,
    Ok(String),
    Err(String),
}

#[component]
pub fn ContractReadPanel(addr: String) -> impl IntoView {
    let (calldata, set_calldata) = signal(String::new());
    let (state, set_state) = signal(ReadState::Idle);

    let on_call = {
        let addr = addr.clone();
        move |_| {
            let cd = calldata.get().trim().to_string();
            let cleaned = cd
                .strip_prefix("0x")
                .or_else(|| cd.strip_prefix("0X"))
                .unwrap_or(&cd)
                .to_string();
            let bytes = match hex::decode(&cleaned) {
                Ok(b) => b,
                Err(e) => {
                    set_state.set(ReadState::Err(format!("hex: {e}")));
                    return;
                }
            };
            let to = addr.clone();
            set_state.set(ReadState::Calling);
            spawn_local(async move {
                #[cfg(target_arch = "wasm32")]
                {
                    use crate::api::{EvmProvider, HttpEvmProvider};
                    let p = HttpEvmProvider::default_for_network();
                    match p.call(&to, &bytes).await {
                        Ok(ret) => set_state.set(ReadState::Ok(format!("0x{}", hex::encode(ret)))),
                        Err(e) => set_state.set(ReadState::Err(format!("{e:?}"))),
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = (to, bytes);
                    set_state.set(ReadState::Err("ssr".into()));
                }
            });
        }
    };

    view! {
        <section class="space-y-3 rounded-2xl border border-zinc-800/40 bg-zinc-900/40 p-4">
            <div class="flex items-center justify-between">
                <h3 class="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                    "Read contract · eth_call"
                </h3>
                <span class="text-[10px] text-zinc-600">"read-only · no signing"</span>
            </div>
            <input
                type="text"
                placeholder="calldata (0x… selector + encoded args)"
                on:input=move |ev| set_calldata.set(event_target_value(&ev))
                prop:value=move || calldata.get()
                class="hex w-full rounded-lg bg-black/40 px-3 py-2 text-xs text-zinc-200 outline-none placeholder-zinc-600"
            />
            <div class="flex flex-wrap items-center gap-3">
                <button
                    type="button"
                    on:click=on_call
                    class="rounded-md border border-amber-500/40 bg-amber-500/10 px-4 py-1.5 text-xs font-semibold text-amber-200 transition hover:bg-amber-500/20"
                >
                    "Call"
                </button>
                <span class="text-xs text-zinc-400">
                    {move || match state.get() {
                        ReadState::Idle => String::new(),
                        ReadState::Calling => "calling…".into(),
                        ReadState::Ok(_) => "ok".into(),
                        ReadState::Err(e) => format!("error · {e}"),
                    }}
                </span>
            </div>
            {move || match state.get() {
                ReadState::Ok(ret) => Some(view! {
                    <pre class="hex max-h-40 overflow-auto rounded-lg bg-black/40 p-3 text-[11px] text-zinc-200">
                        {ret}
                    </pre>
                }),
                _ => None,
            }}
        </section>
    }
}
