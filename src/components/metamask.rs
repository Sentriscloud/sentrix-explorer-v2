//! MetaMask (EIP-1193) connect button.
//!
//! ## What ships today
//!
//! - Click → `eth_requestAccounts` via `window.ethereum`. The wallet
//!   returns the user's accounts; we surface the first one.
//! - Display state: not-connected / connected (truncated address) /
//!   no-provider (e.g., user has no wallet extension).
//!
//! ## What's deferred
//!
//! `eth_sendTransaction`, `eth_signTypedData_v4`, network-add (4902),
//! network-switch (3326), receipt polling. Those are the Dev Lab
//! deploy flow; they go in once the EVM provider trait is wired
//! against a real RPC backend so we have a place to verify receipts.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::i18n::{t, use_lang};

#[derive(Clone, Debug)]
pub enum WalletState {
    Idle,
    NoProvider,
    Connecting,
    Connected(String),
    Error(String),
}

#[component]
pub fn MetaMaskButton() -> impl IntoView {
    let (state, set_state) = signal(WalletState::Idle);
    let lang = use_lang();

    let on_click = move |_| {
        set_state.set(WalletState::Connecting);
        spawn_local(async move {
            match request_accounts().await {
                Ok(Some(addr)) => set_state.set(WalletState::Connected(addr)),
                Ok(None) => set_state.set(WalletState::NoProvider),
                Err(e) => set_state.set(WalletState::Error(e)),
            }
        });
    };

    view! {
        <button
            type="button"
            on:click=on_click
            class="rounded-md border border-zinc-800 bg-zinc-900/40 px-3 py-1.5 text-xs font-medium text-zinc-300 transition hover:border-amber-500/40 hover:text-amber-200"
        >
            {move || render_label(state.get(), lang.get())}
        </button>
    }
}

fn render_label(s: WalletState, lang: crate::i18n::Lang) -> String {
    match s {
        WalletState::Idle => t(lang, "wallet.connect"),
        WalletState::NoProvider => "No wallet".into(),
        WalletState::Connecting => "…".into(),
        WalletState::Connected(addr) => {
            // 0x1234…abcd — keep it scannable in the navbar at any width.
            if addr.len() >= 12 {
                format!("{}…{}", &addr[..6], &addr[addr.len() - 4..])
            } else {
                addr
            }
        }
        WalletState::Error(_) => "Retry".into(),
    }
}

/// Fire-and-await a contract deploy. `from` is the connected account
/// (omit `to` for deploys); `data` is `0x`-prefixed bytecode.
/// Returns the tx hash. Caller is responsible for receipt polling.
#[cfg(target_arch = "wasm32")]
pub async fn send_deploy(from: &str, bytecode_hex: &str) -> Result<String, String> {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let win = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let eth = match js_sys::Reflect::get(&win, &JsValue::from_str("ethereum")) {
        Ok(v) if !v.is_undefined() && !v.is_null() => v,
        _ => return Err("no wallet".into()),
    };

    let req_fn: js_sys::Function = js_sys::Reflect::get(&eth, &JsValue::from_str("request"))
        .map_err(|_| "ethereum.request missing".to_string())?
        .dyn_into()
        .map_err(|_| "ethereum.request not callable".to_string())?;

    // Build params: [{ from, data }] — no `to` ⇒ contract creation.
    let tx = js_sys::Object::new();
    js_sys::Reflect::set(&tx, &JsValue::from_str("from"), &JsValue::from_str(from)).ok();
    js_sys::Reflect::set(
        &tx,
        &JsValue::from_str("data"),
        &JsValue::from_str(bytecode_hex),
    )
    .ok();

    let params = js_sys::Array::new();
    params.push(&tx);

    let arg = js_sys::Object::new();
    js_sys::Reflect::set(
        &arg,
        &JsValue::from_str("method"),
        &JsValue::from_str("eth_sendTransaction"),
    )
    .ok();
    js_sys::Reflect::set(&arg, &JsValue::from_str("params"), &params).ok();

    let promise = req_fn
        .call1(&eth, &arg)
        .map_err(|_| "request rejected".to_string())?;
    let promise: js_sys::Promise = promise
        .dyn_into()
        .map_err(|_| "request did not return a Promise".to_string())?;

    let hash = JsFuture::from(promise)
        .await
        .map_err(|e| format!("rejected: {e:?}"))?;
    hash.as_string()
        .ok_or_else(|| "tx hash not a string".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn send_deploy(_from: &str, _bytecode: &str) -> Result<String, String> {
    Err("ssr".into())
}

#[cfg(target_arch = "wasm32")]
async fn request_accounts() -> Result<Option<String>, String> {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let win = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let eth = match js_sys::Reflect::get(&win, &JsValue::from_str("ethereum")) {
        Ok(v) if !v.is_undefined() && !v.is_null() => v,
        _ => return Ok(None),
    };

    // eth.request({ method: "eth_requestAccounts" }).
    let req_fn = js_sys::Reflect::get(&eth, &JsValue::from_str("request"))
        .map_err(|_| "ethereum.request missing".to_string())?;
    let req_fn: js_sys::Function = req_fn
        .dyn_into()
        .map_err(|_| "ethereum.request not callable".to_string())?;

    let arg = js_sys::Object::new();
    js_sys::Reflect::set(
        &arg,
        &JsValue::from_str("method"),
        &JsValue::from_str("eth_requestAccounts"),
    )
    .ok();

    let promise = req_fn
        .call1(&eth, &arg)
        .map_err(|_| "request rejected".to_string())?;
    let promise: js_sys::Promise = promise
        .dyn_into()
        .map_err(|_| "request did not return a Promise".to_string())?;

    let accounts = JsFuture::from(promise)
        .await
        .map_err(|e| format!("rejected: {e:?}"))?;
    let arr: js_sys::Array = accounts
        .dyn_into()
        .map_err(|_| "accounts not an array".to_string())?;
    if arr.length() == 0 {
        return Ok(None);
    }
    let first = arr.get(0).as_string();
    Ok(first)
}

#[cfg(not(target_arch = "wasm32"))]
async fn request_accounts() -> Result<Option<String>, String> {
    Ok(None)
}
