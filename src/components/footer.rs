//! Footer — links + brand attribution.
//!
//! Lives at the App root so every route shares the same chrome.
//! Uses `ServiceRegistry` for the network-correct external links;
//! protocol/foundation links go to the Sentrix Labs brand surfaces
//! (sentrix-labs GitHub org), not the SentrisCloud product surfaces.

use leptos::prelude::*;

use crate::context::network::ServiceRegistry;
use crate::state::chain::ChainState;

#[component]
pub fn Footer() -> impl IntoView {
    let services = use_context::<ServiceRegistry>().expect("ServiceRegistry context");
    let chain = use_context::<ChainState>().expect("ChainState context");

    view! {
        <footer class="mt-auto border-t border-zinc-800/40 pt-8 pb-6 text-xs text-zinc-500">
            <div class="grid grid-cols-2 gap-8 md:grid-cols-4">
                <Group title="Sentrix Chain">
                    <FooterLink href="https://sentrixchain.com" label="Homepage" />
                    <FooterLink
                        href="https://github.com/sentrix-labs"
                        label="Sentrix Labs · GitHub"
                    />
                    <FooterLink
                        href="https://github.com/sentrix-labs/brand-kit"
                        label="Brand Kit"
                    />
                </Group>

                <Group title="Network">
                    <FooterLink href=services.faucet label="Faucet" />
                    <FooterLink href=services.wallet label="Wallet" />
                    <FooterLink href=services.coinblast label="CoinBlast" />
                </Group>

                <Group title="Tools">
                    <InternalLink href="/" label="Dashboard" />
                    <InternalLink href="/assets" label="Assets" />
                    <InternalLink href="/lab" label="Developer Lab" />
                </Group>

                <Group title="Engine">
                    <span class="block py-0.5 text-zinc-400">"Obsidian Engine"</span>
                    <span class="block py-0.5">"Leptos · Rust + WASM"</span>
                    <span class="block py-0.5">"gRPC-Web wire format"</span>
                </Group>
            </div>

            <div class="mt-8 flex flex-wrap items-center justify-between gap-3 border-t border-zinc-800/40 pt-4 text-[10px] uppercase tracking-[0.18em] text-zinc-600">
                <span>"© 2026 SentrisCloud · Sentrix Chain"</span>
                <span class="flex items-center gap-3">
                    <span>
                        "chain id · "
                        {move || match chain.chain_id.get() {
                            Some(id) => id.to_string(),
                            None => "—".into(),
                        }}
                    </span>
                    <span>"v0.1 · Obsidian Engine"</span>
                </span>
            </div>
        </footer>
    }
}

#[component]
fn Group(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div>
            <div class="eyebrow mb-2 text-zinc-400">{title}</div>
            <div class="space-y-1">{children()}</div>
        </div>
    }
}

#[component]
fn FooterLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a
            href=href
            target="_blank"
            rel="noopener"
            class="block py-0.5 transition hover:text-emerald-500"
        >
            {label}
        </a>
    }
}

#[component]
fn InternalLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a
            href=href
            class="block py-0.5 transition hover:text-emerald-500"
        >
            {label}
        </a>
    }
}
