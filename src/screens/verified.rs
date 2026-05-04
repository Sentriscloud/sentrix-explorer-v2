//! `/verified` — canonical contract registry view.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::identicon::Identicon;
use crate::components::verified_badge::VerifiedBadge;
use crate::state::canonical::{CanonicalContract, CanonicalRegistryState};

#[component]
pub fn VerifiedScreen() -> impl IntoView {
    let registry = use_context::<CanonicalRegistryState>().expect("CanonicalRegistryState context");

    view! {
        <Title text="Sentrix Verified Contracts — Obsidian Engine" />

        <section class="glass-card rounded-2xl p-6">
            <header class="mb-6 flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-bold italic tracking-tighter text-zinc-100">
                        "VERIFIED CONTRACTS"
                    </h2>
                    <p class="mt-1 text-xs text-zinc-500">
                        "Canonical registry · curated by Sentrix Labs"
                    </p>
                </div>
                <span class="status-pill">
                    {move || format!("{} listed", registry.contracts.with(|c| c.len()))}
                </span>
            </header>

            <Show
                when=move || registry.contracts.with(|c| !c.is_empty())
                fallback=|| view! { <RegistryEmpty /> }
            >
                <div class="space-y-2">
                    <For
                        each=move || registry.contracts.get()
                        key=|c| c.address.clone()
                        children=|c: CanonicalContract| view! { <ContractRow c /> }
                    />
                </div>
            </Show>
        </section>
    }
}

#[component]
fn ContractRow(c: CanonicalContract) -> impl IntoView {
    let link = format!("/address/{}", c.address);
    let seed = c
        .address
        .strip_prefix("0x")
        .or_else(|| c.address.strip_prefix("0X"))
        .unwrap_or(&c.address)
        .to_string();
    let short = if c.address.len() >= 12 {
        format!("{}…{}", &c.address[..6], &c.address[c.address.len() - 4..])
    } else {
        c.address.clone()
    };
    view! {
        <a
            href=link
            class="flex items-center justify-between rounded-xl border border-zinc-800/30 bg-zinc-900/40 p-3 text-xs transition-all hover:border-zinc-700"
        >
            <div class="flex items-center gap-3">
                <div class="identicon-frame h-9 w-9 rounded-lg ring-1 ring-zinc-800/80">
                    <Identicon address_hex=seed size=36 />
                </div>
                <div>
                    <div class="font-bold text-zinc-100">{c.name}</div>
                    <div class="hex text-[10px] text-zinc-500">{short} " · " {c.role}</div>
                </div>
            </div>
            <VerifiedBadge status=c.status />
        </a>
    }
}

#[component]
fn RegistryEmpty() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-dashed border-zinc-800 bg-zinc-900/30 p-10 text-center">
            <h3 class="text-sm font-semibold text-zinc-300">
                "Registry awaiting first publication"
            </h3>
            <p class="mx-auto mt-2 max-w-md text-xs text-zinc-500">
                "Canonical contracts (token, treasury, governance, multisig) will appear here once Sentrix Labs publishes the curated registry. Each entry carries a verification anchor and is signed off by the foundation."
            </p>
        </div>
    }
}
