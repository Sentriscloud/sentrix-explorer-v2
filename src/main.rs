//! Dual entry: SSR (axum) on the server, hydration shim on wasm32.
//!
//! `cargo leptos build` produces both targets from this same crate. The
//! browser bundle compiles only the `#[cfg(target_arch = "wasm32")]`
//! arm; native builds compile the SSR arm.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sentrix_explorer_v2::{shell, App};

    let conf = get_configuration(None).expect("invalid Leptos config");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let opts = leptos_options.clone();
            move || shell(opts.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind site_addr");
    eprintln!("obsidian-engine listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("axum serve");
}

#[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
fn main() {
    // Hydration entry comes through `lib::hydrate()` invoked by trunk/leptos
    // index.html; this stub keeps cargo happy for `--target wasm32-*`.
}

#[cfg(all(not(feature = "ssr"), not(target_arch = "wasm32")))]
fn main() {
    // Native build without ssr feature — used by tools that just want
    // to typecheck the lib half.
}
