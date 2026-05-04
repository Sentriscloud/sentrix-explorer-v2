//! Tiny shared helpers — browser sleep, hex helpers, etc.

/// Browser-side sleep via `setTimeout`. Compiled only on wasm32 — the
/// SSR target gets a never-resolving stub so callers can `.await` it
/// without a cfg fence. The SSR `spawn_local` body never polls past
/// the first awaiting future on the server runtime, so the stub is
/// safe to leave dangling.
#[cfg(target_arch = "wasm32")]
pub async fn sleep_ms(ms: i32) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let win = web_sys::window().expect("no window");
        win.set_timeout_with_callback_and_timeout_and_arguments_0(
            &Closure::once_into_js(move || {
                resolve.call0(&JsValue::NULL).ok();
            })
            .unchecked_into::<js_sys::Function>(),
            ms,
        )
        .ok();
    });
    let _ = JsFuture::from(promise).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep_ms(_ms: i32) {
    std::future::pending::<()>().await
}
