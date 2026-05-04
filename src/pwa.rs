//! PWA glue — service worker registration on hydrate.
//!
//! The SW itself is plain JS at `/sw.js`; this module just kicks off
//! `navigator.serviceWorker.register` once the WASM bundle is alive.
//! Failure is silent — a missing or rejected SW shouldn't take down
//! the explorer, and the static-asset cache benefit is opt-in by
//! design (privacy-conscious browsers may decline).

#[cfg(target_arch = "wasm32")]
pub fn register_service_worker() {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let win = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let nav = win.navigator();
    let container = match nav
        .service_worker()
        .dyn_into::<web_sys::ServiceWorkerContainer>()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    leptos::task::spawn_local(async move {
        let promise = container.register("/sw.js");
        let _ = JsFuture::from(promise).await;
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_service_worker() {
    // SSR no-op — only the hydrated browser bundle has navigator.
}
