//! CopyCli — one-click copy of a Sentrix CLI invocation.
//!
//! `srx-cli get block --height 100`, `srx-cli get tx --hash 0x…`, etc.
//! Click triggers `navigator.clipboard.writeText`; on success the
//! button flips to a checkmark for a couple of seconds then resets.
//!
//! SSR fallback: button still renders (so the layout pre-paints
//! correctly), but the click handler is a no-op until the WASM
//! bundle hydrates. No JS-required runtime gates.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::util::sleep_ms;

#[component]
pub fn CopyCli(
    /// The exact CLI string to copy. Build it once at the call site —
    /// e.g. `format!("srx-cli get block --height {h}")`.
    command: String,
) -> impl IntoView {
    let (copied, set_copied) = signal(false);
    let cmd_for_copy = command.clone();

    let on_click = move |_| {
        let cmd = cmd_for_copy.clone();
        spawn_local(async move {
            let ok = write_clipboard(&cmd).await;
            if ok {
                set_copied.set(true);
                sleep_ms(1_800).await;
                set_copied.set(false);
            }
        });
    };

    view! {
        <button
            type="button"
            on:click=on_click
            class="group inline-flex items-center gap-2 rounded-md border border-zinc-800 bg-zinc-900/40 px-3 py-1.5 font-mono text-xs text-zinc-300 transition hover:border-amber-500/40 hover:text-amber-200"
        >
            <span class="hex">{command}</span>
            <span class="border-l border-zinc-700 pl-2 text-[10px] uppercase tracking-wider text-zinc-500 group-hover:text-amber-300">
                {move || if copied.get() { "Copied ✓" } else { "Copy CLI" }}
            </span>
        </button>
    }
}

#[cfg(target_arch = "wasm32")]
async fn write_clipboard(text: &str) -> bool {
    use wasm_bindgen_futures::JsFuture;

    let Some(win) = web_sys::window() else {
        return false;
    };
    let nav = win.navigator();
    let clip = nav.clipboard();
    JsFuture::from(clip.write_text(text)).await.is_ok()
}

#[cfg(not(target_arch = "wasm32"))]
async fn write_clipboard(_text: &str) -> bool {
    false
}
