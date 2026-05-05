use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::lang_switcher::LanguageSwitcher;
use crate::components::metamask::MetaMaskButton;
use crate::config::Network;
use crate::context::network::ServiceRegistry;
use crate::i18n::{t, use_lang};
use crate::state::feed::BlockFeedState;

#[component]
pub fn Navbar() -> impl IntoView {
    let network = Network::current();
    let services = use_context::<ServiceRegistry>().expect("ServiceRegistry context not provided");

    let lang = use_lang();

    let (badge_class, badge_text) = match network {
        Network::Mainnet => (
            "border-sentrix-gold/30 bg-sentrix-gold/10 text-sentrix-gold",
            "MAINNET",
        ),
        Network::Testnet => ("border-sky-400/30 bg-sky-400/10 text-sky-300", "TESTNET"),
    };

    let toggle_target = services.sibling_explorer;
    let toggle_key = match network {
        Network::Mainnet => "nav.switch_to_testnet",
        Network::Testnet => "nav.switch_to_mainnet",
    };

    // Mobile hamburger toggles a slide-down panel of the same nav
    // links. Toggle signal lives on the client; SSR pre-renders the
    // panel hidden so hydration sees the same DOM either way.
    let nav_open = RwSignal::new(false);

    // Cross-subdomain navigation — flushes browser memory + rehydrates
    // the bundle compiled for the *other* network. Same outcome as
    // setting `window.location.href` directly, but routed through
    // Leptos so SSR and CSR agree on the markup.
    view! {
        <header class="sticky top-0 z-30 -mx-4 mb-8 border-b border-zinc-800/60 bg-zinc-950/95">
        <div class="flex flex-wrap items-center justify-between gap-4 px-4 py-3">
            <div class="flex items-center gap-6">
                <a href=services.explorer class="flex items-center gap-3 group">
                    <BrandMark />
                    <div class="flex flex-col leading-tight">
                        <span class="font-serif text-xl font-semibold tracking-tight text-zinc-100 transition-colors group-hover:text-sentrix-gold">
                            "Sentrix"
                        </span>
                        <span class="eyebrow text-zinc-500">
                            "Obsidian Engine · Rust + WASM"
                        </span>
                    </div>
                </a>

                <nav class="hidden items-center gap-1 text-sm md:flex">
                    <NavTab href="/" label_key="nav.dashboard" />
                    <NavTab href="/assets" label_key="nav.assets" />
                    <NavTab href="/lab" label_key="nav.lab" />
                    <NavTab href="/contracts" label_key="nav.contracts" />
                </nav>
            </div>

            <nav class="flex flex-wrap items-center justify-end gap-2 text-sm">
                <ConnectionStatus />
                <CommandHint />
                <ExternalLink href=services.faucet label_key="nav.faucet" />
                <ExternalLink href=services.wallet label_key="nav.wallet" />
                <ExternalLink href=services.coinblast label_key="nav.coinblast" />
                <MetaMaskButton />
                <ThemeToggle />
                <LanguageSwitcher />

                <span class=format!(
                    "rounded-md border px-2.5 py-1 font-mono text-[10px] uppercase tracking-[0.18em] {badge_class}"
                )>{badge_text}</span>

                <a
                    href=toggle_target
                    title=move || t(lang.get(), toggle_key)
                    class="rounded-md border border-sentrix-gold/40 bg-sentrix-gold/10 px-3 py-1.5 text-xs font-semibold tracking-wide text-sentrix-gold transition hover:border-sentrix-gold/70 hover:bg-sentrix-gold/15"
                >
                    {move || t(lang.get(), toggle_key)}
                </a>

                <button
                    type="button"
                    aria-label="Toggle navigation"
                    on:click=move |_| nav_open.update(|o| *o = !*o)
                    class="ml-1 inline-flex h-9 w-9 items-center justify-center rounded-md border border-zinc-800 bg-zinc-900/40 text-zinc-300 transition hover:border-sentrix-gold/40 hover:text-sentrix-gold md:hidden"
                >
                    {move || if nav_open.get() {
                        view! {
                            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M18 6L6 18M6 6l12 12" />
                            </svg>
                        }.into_any()
                    } else {
                        view! {
                            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M3 6h18M3 12h18M3 18h18" />
                            </svg>
                        }.into_any()
                    }}
                </button>
            </nav>
        </div>

        <Show when=move || nav_open.get() fallback=|| ()>
            <div class="border-t border-zinc-800/60 px-4 py-3 md:hidden">
                <nav class="flex flex-col gap-1 text-sm">
                    <MobileNavLink href="/" label_key="nav.dashboard" close=nav_open />
                    <MobileNavLink href="/assets" label_key="nav.assets" close=nav_open />
                    <MobileNavLink href="/lab" label_key="nav.lab" close=nav_open />
                    <MobileNavLink href="/contracts" label_key="nav.contracts" close=nav_open />
                </nav>
            </div>
        </Show>
        </header>
    }
}

#[component]
fn MobileNavLink(href: &'static str, label_key: &'static str, close: RwSignal<bool>) -> impl IntoView {
    let lang = use_lang();
    view! {
        <a
            href=href
            on:click=move |_| close.set(false)
            class="rounded-md px-3 py-2 text-zinc-300 transition hover:bg-zinc-900/60 hover:text-sentrix-gold"
        >
            {move || t(lang.get(), label_key)}
        </a>
    }
}

#[component]
fn NavTab(href: &'static str, label_key: &'static str) -> impl IntoView {
    let lang = use_lang();
    view! {
        <A
            href=href
            attr:class="rounded-md px-3 py-1.5 text-zinc-400 transition hover:text-zinc-100"
        >
            {move || t(lang.get(), label_key)}
        </A>
    }
}

#[component]
fn ExternalLink(href: &'static str, label_key: &'static str) -> impl IntoView {
    let lang = use_lang();
    view! {
        <a
            href=href
            target="_blank"
            rel="noopener"
            class="rounded-md border border-zinc-800 bg-zinc-900/40 px-3 py-1.5 text-zinc-300 transition hover:border-sentrix-gold/40 hover:text-sentrix-gold"
        >
            {move || t(lang.get(), label_key)}
        </a>
    }
}

#[component]
fn ConnectionStatus() -> impl IntoView {
    let feed = use_context::<BlockFeedState>().expect("BlockFeedState context");

    // Map feed.status (free-form &'static str) to a state class. The
    // state strings are produced by `state::feed`; matching by
    // `contains` rather than equality gives us forward-compat against
    // small wording tweaks there.
    let dot_class = move || {
        let s = feed.status.get();
        if s.contains("error") {
            "bg-rose-500"
        } else if s.contains("polling") || s.contains("retry") {
            "bg-amber-400"
        } else if s.contains("streaming") || s.contains("live") {
            "bg-emerald-400"
        } else {
            "bg-zinc-500"
        }
    };

    view! {
        <span class="hidden items-center gap-1.5 rounded-md border border-zinc-800 bg-zinc-900/40 px-2.5 py-1 text-[11px] text-zinc-400 sm:inline-flex">
            <span class=move || format!("h-1.5 w-1.5 rounded-full {}", dot_class())></span>
            <span>{move || feed.status.get()}</span>
        </span>
    }
}

#[component]
fn BrandMark() -> impl IntoView {
    // Inline SVG of the canonical sentrix-mark-tight from
    // brand-kit/svg. Bronze outline + filled inner diamond + four
    // gold cardinal nodes. Inlined (rather than `<img src=…>`) so it
    // tints with the surrounding theme on Solar mode without an
    // extra fetch.
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="320 320 384 384"
            class="h-8 w-8 shrink-0"
            shape-rendering="geometricPrecision"
            aria-label="Sentrix Chain"
        >
            <polygon
                points="512,340 685,513 512,686 339,513"
                fill="none"
                stroke="#8A5A11"
                stroke-width="12"
                stroke-linejoin="miter"
                stroke-miterlimit="10"
            />
            <polygon
                points="512,438 586,512 512,586 438,512"
                fill="#8A5A11"
            />
            <circle cx="512" cy="340" r="11" fill="#DBC17F" />
            <circle cx="685" cy="513" r="11" fill="#DBC17F" />
            <circle cx="512" cy="686" r="11" fill="#DBC17F" />
            <circle cx="339" cy="513" r="11" fill="#DBC17F" />
        </svg>
    }
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let on_click = move |_| {
        let _ = crate::theme::toggle();
    };
    view! {
        <button
            type="button"
            on:click=on_click
            title="Toggle Obsidian / Solar"
            class="rounded-md border border-zinc-800 bg-zinc-900/40 px-3 py-1.5 text-zinc-300 transition hover:border-sentrix-gold/40 hover:text-sentrix-gold"
        >
            // Sun + moon glyphs in one toggle. Solar shows sun-only,
            // Obsidian shows the crescent — the parent .solar class
            // swaps via display:none on the unselected glyph.
            <span class="solar-only hidden">"☀"</span>
            <span class="obsidian-only">"☾"</span>
        </button>
    }
}

#[component]
fn CommandHint() -> impl IntoView {
    // Static hint — keypress handler lives in `command_palette`. Mac
    // shows ⌘K, others show Ctrl+K; we don't UA-sniff, just print
    // both glyphs separated by a slash so it's unambiguous.
    view! {
        <span class="hidden items-center gap-1 rounded-md border border-zinc-800 bg-zinc-900/40 px-2.5 py-1 text-[11px] text-zinc-400 sm:inline-flex">
            <kbd class="rounded border border-zinc-700 bg-zinc-800 px-1 py-0 font-mono text-[10px]">"⌘"</kbd>
            <kbd class="rounded border border-zinc-700 bg-zinc-800 px-1 py-0 font-mono text-[10px]">"K"</kbd>
            <span class="ml-1">"search"</span>
        </span>
    }
}
