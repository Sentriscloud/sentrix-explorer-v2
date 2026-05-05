//! Obsidian Engine — Sentrix Explorer V2 (Leptos + WASM).
//!
//! Browser-side WASM bundle subscribes directly to the chain's gRPC-Web
//! endpoint (`grpc.sentrixchain.com:443`). No JSON-RPC, no Next.js
//! middleware: the proto types in `grpc::pb` are the single source of
//! truth from wire to UI.

#![forbid(unsafe_code)]
// Leptos's `view!` macro generates massively-nested generic types as
// it composes children; the default 128-depth limit overflows on
// pages like the Dashboard that compose StatsPanel + LiveBlockFeed +
// Mempool + ValidatorActivity in one tree. 512 is the empirically
// observed safe ceiling.
#![recursion_limit = "1024"]

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, StaticSegment,
};

pub mod api;
pub mod components;
pub mod config;
pub mod context;
pub mod grpc;
pub mod i18n;
pub mod pwa;
pub mod screens;
pub mod state;
pub mod theme;
pub mod util;

/// Resolved at compile time from `SENTRIX_NETWORK` + per-network override
/// env vars. See `config::grpc_endpoint`.
pub const GRPC_ENDPOINT: &str = config::grpc_endpoint();

/// CSS bundle version. Bump when tailwind.css shape changes — drives
/// the `?v=N` query param on the `<Stylesheet>` href so CDN/edge
/// caches don't pin a stale build.
const CSS_BUMP: u32 = 2;

/// Document shell — only used by SSR; CSR mounts directly on `<body>`.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="theme-color" content="#0A0A0C" />
                <link rel="manifest" href="/manifest.json" />
                <link rel="icon" type="image/svg+xml" href="/icon.svg" />
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
                <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
                <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
                <link rel="mask-icon" href="/safari-pinned-tab.svg" color="#8A5A11" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="min-h-screen bg-zinc-950 text-zinc-100 transition-colors">
                <components::testnet_banner::TestnetBanner />
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    // Resolved external-service URLs (faucet/wallet/coinblast/sibling
    // explorer). Compile-time const value, but routed through context
    // so deeply-nested view code stays pluggable for tests / mocks.
    provide_context(context::network::services());

    // Active language signal — read from localStorage, pushed back on
    // change. Components grab it via `i18n::use_lang()`.
    i18n::provide_lang_signal();

    // Single gRPC subscriptions, multi-consumer signals.
    state::feed::provide_block_feed();
    state::mempool::provide_mempool();
    state::gas::provide_gas_price();
    state::chain::provide_chain_state();
    state::deploys::provide_deploy_history();
    state::canonical::provide_canonical_registry();
    state::network::provide_network_signal();

    let (title, description) = match config::Network::current() {
        config::Network::Mainnet => (
            "Sentrix Explorer — Obsidian Engine",
            "Real-time block explorer for the Sentrix L1 — Rust + WASM, gRPC-Web wire format.",
        ),
        config::Network::Testnet => (
            "Sentrix Testnet Scan — Obsidian Engine",
            "Sentrix testnet block explorer — assets have no real-world value.",
        ),
    };

    view! {
        // Version query bypasses any stale CDN cache without needing
        // a CF API purge. Bump CSS_BUMP whenever the bundled CSS
        // shape changes. The HTML itself isn't long-cached by Caddy
        // so the new ?v= is picked up on the next page load.
        <Stylesheet id="leptos" href=format!("/pkg/sentrix-explorer-v2.css?v={}", CSS_BUMP) />
        <Title text=title />
        <Meta name="description" content=description />
        <Meta property="og:type" content="website" />
        <Meta property="og:title" content=title />
        <Meta property="og:description" content=description />
        <Meta property="og:image" content="https://scan.sentriscloud.com/icon.svg" />
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:title" content=title />
        <Meta name="twitter:description" content=description />
        <Meta name="twitter:image" content="https://scan.sentriscloud.com/icon.svg" />

        <Router>
            <main class="container mx-auto flex min-h-screen max-w-6xl flex-col px-4 py-8">
                <components::navbar::Navbar />
                <Routes fallback=screens::not_found::NotFoundScreen>
                    <Route path=StaticSegment("") view=screens::dashboard::Dashboard />
                    <Route path=StaticSegment("assets") view=screens::assets::AssetsScreen />
                    <Route path=StaticSegment("lab") view=screens::lab::LabScreen />
                    <Route
                        path=StaticSegment("contracts")
                        view=screens::contracts::ContractsScreen
                    />
                    <Route
                        path=StaticSegment("verified")
                        view=screens::verified::VerifiedScreen
                    />
                    <Route
                        path=(StaticSegment("block"), ParamSegment("height"))
                        view=screens::block_detail::BlockDetailScreen
                    />
                    <Route
                        path=(StaticSegment("tx"), ParamSegment("hash"))
                        view=screens::tx_detail::TxDetailScreen
                    />
                    <Route
                        path=(StaticSegment("address"), ParamSegment("addr"))
                        view=screens::address_detail::AddressDetailScreen
                    />
                </Routes>
                <components::footer::Footer />
            </main>
            <components::command_palette::CommandPalette />
        </Router>
    }
}

/// Browser entry — wired up in `main.rs` under `#[cfg(target_arch = "wasm32")]`.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    pwa::register_service_worker();
    theme::apply_persisted_theme();
    leptos::mount::hydrate_body(App);
}
