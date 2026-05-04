use leptos::prelude::*;
use leptos_meta::Title;

use crate::i18n::{t, use_lang};

#[component]
pub fn NotFoundScreen() -> impl IntoView {
    let lang = use_lang();
    view! {
        <Title text="Not Found · Sentrix Explorer" />

        <section class="glass-card mx-auto max-w-lg rounded-2xl p-10 text-center">
            <div class="mx-auto mb-6 h-12 w-12">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="320 320 384 384"
                    class="h-12 w-12 opacity-40"
                    aria-hidden="true"
                >
                    <polygon
                        points="512,340 685,513 512,686 339,513"
                        fill="none"
                        stroke="#8A5A11"
                        stroke-width="12"
                        stroke-linejoin="miter"
                    />
                    <polygon
                        points="512,438 586,512 512,586 438,512"
                        fill="#8A5A11"
                    />
                </svg>
            </div>
            <h1 class="font-mono text-3xl font-bold text-zinc-100">
                {move || t(lang.get(), "not_found.title")}
            </h1>
            <p class="mt-2 text-sm text-zinc-400">
                {move || t(lang.get(), "not_found.body")}
            </p>
            <a
                href="/"
                class="mt-6 inline-block rounded-md border border-amber-500/40 bg-amber-500/10 px-4 py-1.5 text-xs font-semibold text-amber-200 transition hover:border-amber-400 hover:bg-amber-500/20"
            >
                {move || t(lang.get(), "not_found.back")}
            </a>
        </section>
    }
}
