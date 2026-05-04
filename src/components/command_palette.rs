//! Cmd+K / Ctrl+K command palette.
//!
//! ## Routing logic
//!
//! Input strings are classified by shape, not by guessing:
//!   - all digits, fits in u64       → block height       → `/block/<h>`
//!   - 64 hex chars (32 bytes)        → block or tx hash   → `/tx/<hash>`
//!     (we route to the tx page; the page falls back to a block lookup
//!     if the hash isn't in the tx index)
//!   - 40 hex chars (20 bytes)        → wallet address     → `/address/<addr>`
//!   - any other token                → fuzzy local search  (RWA assets,
//!     once the registry is populated)
//!
//! Routes don't all exist yet — the placeholder dashboard is the only
//! live page today. `/block/<h>` etc. land in the 404 fallback. The
//! palette still works as a router-aware UX surface so when those
//! pages ship, dispatch is already wired.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    BlockHeight(u64),
    Hash(String),
    Address(String),
    Unknown,
}

fn classify(input: &str) -> Target {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Target::Unknown;
    }

    if let Ok(h) = trimmed.parse::<u64>() {
        return Target::BlockHeight(h);
    }

    let cleaned = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    let is_hex = !cleaned.is_empty() && cleaned.bytes().all(|b| b.is_ascii_hexdigit());
    if is_hex {
        match cleaned.len() {
            64 => return Target::Hash(cleaned.to_lowercase()),
            40 => return Target::Address(cleaned.to_lowercase()),
            _ => {}
        }
    }
    Target::Unknown
}

fn target_path(t: &Target) -> Option<String> {
    match t {
        Target::BlockHeight(h) => Some(format!("/block/{h}")),
        Target::Hash(h) => Some(format!("/tx/0x{h}")),
        Target::Address(a) => Some(format!("/address/0x{a}")),
        Target::Unknown => None,
    }
}

fn target_label(t: &Target) -> &'static str {
    match t {
        Target::BlockHeight(_) => "Block height",
        Target::Hash(_) => "Hash (32-byte)",
        Target::Address(_) => "Wallet address (20-byte)",
        Target::Unknown => "—",
    }
}

#[component]
pub fn CommandPalette() -> impl IntoView {
    let (open, set_open) = signal(false);
    let (query, set_query) = signal(String::new());

    register_hotkey(set_open);

    let target = Memo::new(move |_| query.with(|q| classify(q)));
    let label = Memo::new(move |_| target.with(target_label));
    let path = Memo::new(move |_| target.with(target_path));

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(p) = path.get() {
            set_open.set(false);
            set_query.set(String::new());
            let nav = use_navigate();
            nav(&p, NavigateOptions::default());
        }
    };

    view! {
        <Show when=move || open.get() fallback=|| ()>
            <div
                class="fixed inset-0 z-50 flex items-start justify-center bg-black/70 px-4 pt-24 backdrop-blur-sm"
                on:click=move |_| set_open.set(false)
            >
                <form
                    on:submit=on_submit
                    on:click=|ev| ev.stop_propagation()
                    class="glass-card w-full max-w-xl rounded-2xl p-2"
                >
                    <input
                        type="text"
                        autofocus=true
                        placeholder="Search blocks · txs · addresses · assets…"
                        prop:value=move || query.get()
                        on:input=move |ev| set_query.set(event_target_value(&ev))
                        class="w-full rounded-xl bg-transparent px-4 py-3 font-mono text-sm text-zinc-100 placeholder-zinc-500 outline-none"
                    />
                    <div class="flex items-center justify-between border-t border-zinc-800/60 px-4 py-2 text-[11px] text-zinc-500">
                        <span>"detected: " <span class="text-zinc-300">{move || label.get()}</span></span>
                        <span>
                            <kbd class="rounded border border-zinc-700 bg-zinc-800 px-1.5 py-0.5 text-[10px]">"↵"</kbd>
                            " jump  ·  "
                            <kbd class="rounded border border-zinc-700 bg-zinc-800 px-1.5 py-0.5 text-[10px]">"esc"</kbd>
                            " close"
                        </span>
                    </div>
                </form>
            </div>
        </Show>
    }
}

#[cfg(target_arch = "wasm32")]
fn register_hotkey(set_open: WriteSignal<bool>) {
    use leptos::task::spawn_local;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    spawn_local(async move {
        let win = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let doc = match win.document() {
            Some(d) => d,
            None => return,
        };

        // We intentionally leak the closure — it lives for the entire
        // session, same as the document, so dropping it would just
        // race the page lifetime.
        let cb =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                let k = ev.key();
                let combo_open = (ev.meta_key() || ev.ctrl_key()) && k == "k";
                let combo_close = k == "Escape";
                if combo_open {
                    ev.prevent_default();
                    set_open.update(|o| *o = !*o);
                } else if combo_close {
                    set_open.set(false);
                }
            });

        doc.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .ok();
        cb.forget();
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn register_hotkey(_set_open: WriteSignal<bool>) {
    // No keyboard on the SSR side; the palette renders closed and the
    // browser-side hydration arms the listener.
}
