//! `/lab` — Developer Lab.
//!
//! Two panels: a Solidity source editor (informational textarea) and a
//! bytecode field that's the actual deployment payload. The deploy
//! button wires `eth_sendTransaction` via MetaMask — no `to`, just
//! `from` + `data` = contract creation. Returns the tx hash on accept.
//!
//! Compiler bridge (solc.js worker) is the next step; until then the
//! source editor is a sketchpad and bytecode is paste-in. Deploying
//! pre-compiled bytecode (e.g. from `forge build`, hardhat, remix) is
//! the same eth_sendTransaction call either way, so the deploy path
//! is real now and the compiler slots in alongside later.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;

use crate::components::copy_cli::CopyCli;
use crate::components::metamask::{send_deploy, MetaMaskButton};
use crate::i18n::{t, use_lang};
use crate::state::deploys::{now_ms, record_deploy, DeployHistoryState, DeployRecord};

// `Ok` is only constructed under `#[cfg(target_arch = "wasm32")]`;
// silence the SSR-side dead-code lint without disabling on the
// whole module.
#[allow(dead_code)]
#[derive(Clone, Debug)]
enum SimState {
    Idle,
    Calling,
    Ok(String),
    Err(String),
}

const SOLIDITY_SAMPLE: &str = "// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Hello {
    string public greeting = \"Hello, Sentrix\";

    function setGreeting(string calldata g) external {
        greeting = g;
    }
}
";

#[derive(Clone, Debug)]
enum DeployState {
    Idle,
    NeedsConnect,
    Sending,
    Sent(String),
    Error(String),
}

#[component]
pub fn LabScreen() -> impl IntoView {
    let lang = use_lang();
    let (source, set_source) = signal(SOLIDITY_SAMPLE.to_string());
    let (bytecode, set_bytecode) = signal(String::new());
    let (from_addr, set_from_addr) = signal::<Option<String>>(None);
    let (deploy, set_deploy) = signal(DeployState::Idle);

    // Read the connected account once on click — we don't keep a long-
    // lived subscription to MetaMask state here. Each deploy attempt
    // re-asks for accounts so a switched wallet picks up automatically.
    let history = use_context::<DeployHistoryState>().expect("DeployHistoryState context");

    let on_deploy = move |_| {
        let bc = bytecode.get().trim().to_string();
        if bc.is_empty() {
            set_deploy.set(DeployState::Error("paste bytecode first".into()));
            return;
        }
        let bc_with_prefix = if bc.starts_with("0x") || bc.starts_with("0X") {
            bc
        } else {
            format!("0x{bc}")
        };
        let bytecode_len = bc_with_prefix
            .strip_prefix("0x")
            .map(|s| s.len() / 2)
            .unwrap_or(0);
        set_deploy.set(DeployState::Sending);
        spawn_local(async move {
            let from = match read_first_account().await {
                Ok(Some(a)) => {
                    set_from_addr.set(Some(a.clone()));
                    a
                }
                _ => {
                    set_deploy.set(DeployState::NeedsConnect);
                    return;
                }
            };
            match send_deploy(&from, &bc_with_prefix).await {
                Ok(hash) => {
                    record_deploy(
                        history,
                        DeployRecord {
                            tx_hash: hash.clone(),
                            bytecode_len,
                            submitted_ms: now_ms(),
                            from: from.clone(),
                        },
                    );
                    set_deploy.set(DeployState::Sent(hash));
                }
                Err(e) => set_deploy.set(DeployState::Error(e)),
            }
        });
    };

    view! {
        <Title text="Sentrix Developer Lab — Obsidian Engine" />

        <section class="space-y-6">
            <header class="flex flex-wrap items-end justify-between gap-3">
                <div>
                    <h1 class="text-xl font-bold italic tracking-tighter text-zinc-100">
                        {move || t(lang.get(), "lab.title")}
                    </h1>
                    <p class="mt-1 text-xs text-zinc-500">
                        {move || t(lang.get(), "lab.subtitle")}
                    </p>
                </div>
                <MetaMaskButton />
            </header>

            <CompilerNotice />

            <div class="space-y-4">
                <div>
                    <label class="mb-2 block text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                        "Solidity source · sketchpad"
                    </label>
                    <div class="glass-card rounded-2xl p-2">
                        <textarea
                            spellcheck="false"
                            class="hex h-72 w-full resize-none rounded-xl bg-black/40 p-4 text-xs leading-relaxed text-zinc-200 outline-none"
                            on:input=move |ev| set_source.set(event_target_value(&ev))
                            prop:value=move || source.get()
                        />
                    </div>
                </div>

                <div>
                    <label class="mb-2 block text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                        "Bytecode · paste compiled output (forge / remix / hardhat)"
                    </label>
                    <div class="glass-card rounded-2xl p-2">
                        <textarea
                            spellcheck="false"
                            placeholder="0x6080604052..."
                            class="hex h-32 w-full resize-none rounded-xl bg-black/40 p-4 text-xs leading-relaxed text-zinc-200 outline-none"
                            on:input=move |ev| set_bytecode.set(event_target_value(&ev))
                            prop:value=move || bytecode.get()
                        />
                    </div>
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-2">
                <DisabledButton label_key="lab.compile" />
                <button
                    type="button"
                    on:click=on_deploy
                    class="rounded-md border border-amber-500/40 bg-amber-500/10 px-4 py-1.5 text-xs font-semibold text-amber-200 transition hover:border-amber-400 hover:bg-amber-500/20"
                >
                    {move || t(lang.get(), "lab.deploy")}
                </button>
                <DeployStatus state=deploy />
                <span class="ml-auto text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                    {move || from_addr.get().map(|a| format!("from {a}…"))}
                </span>
            </div>

            <Simulator />

            <footer class="border-t border-zinc-800/40 pt-4">
                <div class="mb-2 text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                    "Sentrix CLI"
                </div>
                <CopyCli command="srx-cli evm deploy --bytecode <stdin>".to_string() />
            </footer>
        </section>
    }
}

#[component]
fn Simulator() -> impl IntoView {
    let (to, set_to) = signal(String::new());
    let (data, set_data) = signal(String::new());
    let (state, set_state) = signal(SimState::Idle);

    let on_run = move |_| {
        let to_addr = to.get().trim().to_string();
        let calldata = data.get().trim().to_string();
        if to_addr.is_empty() {
            set_state.set(SimState::Err("missing 'to' address".into()));
            return;
        }
        // Strip 0x for the bytes parse step; the EVM provider re-adds.
        let calldata_clean = calldata
            .strip_prefix("0x")
            .or_else(|| calldata.strip_prefix("0X"))
            .unwrap_or(&calldata)
            .to_string();
        let bytes = match hex::decode(&calldata_clean) {
            Ok(b) => b,
            Err(e) => {
                set_state.set(SimState::Err(format!("calldata hex: {e}")));
                return;
            }
        };
        let to_for_call = if to_addr.starts_with("0x") || to_addr.starts_with("0X") {
            to_addr
        } else {
            format!("0x{to_addr}")
        };
        set_state.set(SimState::Calling);
        spawn_local(async move {
            #[cfg(target_arch = "wasm32")]
            {
                use crate::api::{EvmProvider, HttpEvmProvider};
                let p = HttpEvmProvider::default_for_network();
                match p.call(&to_for_call, &bytes).await {
                    Ok(ret) => set_state.set(SimState::Ok(format!("0x{}", hex::encode(ret)))),
                    Err(e) => set_state.set(SimState::Err(format!("{e:?}"))),
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (to_for_call, bytes);
                set_state.set(SimState::Err("ssr".into()));
            }
        });
    };

    view! {
        <div class="space-y-3 rounded-2xl border border-zinc-800/40 bg-zinc-900/40 p-4">
            <div class="flex items-center justify-between">
                <h3 class="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                    "Tx Simulator · eth_call"
                </h3>
                <span class="text-[10px] text-zinc-600">"read-only · no signing"</span>
            </div>
            <div class="grid gap-2 md:grid-cols-2">
                <input
                    type="text"
                    placeholder="to (0x…)"
                    on:input=move |ev| set_to.set(event_target_value(&ev))
                    prop:value=move || to.get()
                    class="hex w-full rounded-lg bg-black/40 px-3 py-2 text-xs text-zinc-200 outline-none placeholder-zinc-600"
                />
                <input
                    type="text"
                    placeholder="calldata (0x…)"
                    on:input=move |ev| set_data.set(event_target_value(&ev))
                    prop:value=move || data.get()
                    class="hex w-full rounded-lg bg-black/40 px-3 py-2 text-xs text-zinc-200 outline-none placeholder-zinc-600"
                />
            </div>
            <div class="flex flex-wrap items-center gap-3">
                <button
                    type="button"
                    on:click=on_run
                    class="rounded-md border border-amber-500/40 bg-amber-500/10 px-4 py-1.5 text-xs font-semibold text-amber-200 transition hover:border-amber-400 hover:bg-amber-500/20"
                >
                    "Simulate"
                </button>
                <span class="text-xs text-zinc-400">
                    {move || match state.get() {
                        SimState::Idle => String::new(),
                        SimState::Calling => "calling…".into(),
                        SimState::Ok(_) => "ok".into(),
                        SimState::Err(e) => format!("error · {e}"),
                    }}
                </span>
            </div>
            {move || match state.get() {
                SimState::Ok(ret) => Some(view! {
                    <pre class="hex max-h-48 overflow-auto rounded-lg bg-black/40 p-3 text-[11px] text-zinc-200">
                        {ret}
                    </pre>
                }),
                _ => None,
            }}
        </div>
    }
}

#[component]
fn CompilerNotice() -> impl IntoView {
    let lang = use_lang();
    view! {
        <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-4 text-xs text-amber-200/80">
            {move || t(lang.get(), "lab.editor_pending")}
        </div>
    }
}

#[component]
fn DisabledButton(label_key: &'static str) -> impl IntoView {
    let lang = use_lang();
    view! {
        <button
            type="button"
            disabled=true
            title="Compiler bridge pending"
            class="cursor-not-allowed rounded-md border border-zinc-800 bg-zinc-900/40 px-3 py-1.5 text-xs font-medium text-zinc-500"
        >
            {move || t(lang.get(), label_key)}
        </button>
    }
}

#[component]
fn DeployStatus(state: ReadSignal<DeployState>) -> impl IntoView {
    view! {
        <span class="text-xs text-zinc-400">
            {move || match state.get() {
                DeployState::Idle => String::new(),
                DeployState::NeedsConnect => "connect wallet first".into(),
                DeployState::Sending => "awaiting wallet…".into(),
                DeployState::Sent(h) => format!("sent · {}", short_hash(&h)),
                DeployState::Error(e) => format!("error · {e}"),
            }}
        </span>
    }
}

fn short_hash(h: &str) -> String {
    if h.len() >= 14 {
        format!("{}…{}", &h[..10], &h[h.len() - 4..])
    } else {
        h.into()
    }
}

#[cfg(target_arch = "wasm32")]
async fn read_first_account() -> Result<Option<String>, String> {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let win = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let eth = match js_sys::Reflect::get(&win, &JsValue::from_str("ethereum")) {
        Ok(v) if !v.is_undefined() && !v.is_null() => v,
        _ => return Ok(None),
    };
    let req_fn: js_sys::Function = js_sys::Reflect::get(&eth, &JsValue::from_str("request"))
        .map_err(|_| "ethereum.request missing".to_string())?
        .dyn_into()
        .map_err(|_| "ethereum.request not callable".to_string())?;

    let arg = js_sys::Object::new();
    js_sys::Reflect::set(
        &arg,
        &JsValue::from_str("method"),
        &JsValue::from_str("eth_requestAccounts"),
    )
    .ok();

    let promise: js_sys::Promise = req_fn
        .call1(&eth, &arg)
        .map_err(|_| "request rejected".to_string())?
        .dyn_into()
        .map_err(|_| "not a promise".to_string())?;

    let accounts = JsFuture::from(promise)
        .await
        .map_err(|e| format!("rejected: {e:?}"))?;
    let arr: js_sys::Array = accounts
        .dyn_into()
        .map_err(|_| "accounts not an array".to_string())?;
    if arr.length() == 0 {
        return Ok(None);
    }
    Ok(arr.get(0).as_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_first_account() -> Result<Option<String>, String> {
    Ok(None)
}
