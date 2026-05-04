//! 3-way language toggle. Click cycles En → Id → Zh → En; selection
//! persists to localStorage via `i18n::persist_lang`.

use leptos::prelude::*;

use crate::i18n::{persist_lang, use_lang, Lang};

#[component]
pub fn LanguageSwitcher() -> impl IntoView {
    let lang = use_lang();

    let cycle = move |_| {
        lang.update(|l| {
            *l = match *l {
                Lang::En => Lang::Id,
                Lang::Id => Lang::Zh,
                Lang::Zh => Lang::En,
            };
            persist_lang(*l);
        });
    };

    view! {
        <button
            type="button"
            on:click=cycle
            title=move || lang.get().full_label()
            class="rounded-md border border-zinc-800 bg-zinc-900/40 px-2.5 py-1.5 text-xs font-semibold text-zinc-300 transition hover:border-amber-500/40 hover:text-amber-200"
        >
            {move || lang.get().label()}
        </button>
    }
}
