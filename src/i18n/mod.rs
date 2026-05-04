//! i18n — three-locale string lookup.
//!
//! ## Layout
//!
//! - `locales/{en,id,zh}.json` — translation tables, hand-edited.
//! - `Lang` enum — exhaustive list of supported locales.
//! - `t(lang, key)` — lookup with key-as-fallback for missing keys.
//! - `provide_lang_signal()` — reads `localStorage["sentrix-lang"]`,
//!   provides an `RwSignal<Lang>` via context.
//! - `LanguageSwitcher` (in `components::lang_switcher`) — three-way
//!   button that cycles + persists.
//!
//! ## Why JSON parsed at runtime
//!
//! Compile-time `&[(&str, &str)]` arrays would be marginally faster
//! but force translators to edit Rust source. Keeping JSON keeps
//! translation work portable to external tools (Crowdin, etc.) when
//! the registry of translators grows beyond one operator.
//!
//! Parse cost: each locale is ~3 KB → microseconds, once, on first
//! lookup. Stored in `OnceLock<HashMap>` so repeat lookups are
//! amortised hash hits.

use std::collections::HashMap;
use std::sync::OnceLock;

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Id,
    Zh,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::En, Lang::Id, Lang::Zh];

    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Id => "id",
            Self::Zh => "zh",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::En => "EN",
            Self::Id => "ID",
            Self::Zh => "中",
        }
    }

    pub fn full_label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Id => "Bahasa Indonesia",
            Self::Zh => "中文",
        }
    }

    pub fn from_code(s: &str) -> Self {
        match s {
            "id" => Self::Id,
            "zh" => Self::Zh,
            _ => Self::En,
        }
    }

    fn raw(self) -> &'static str {
        match self {
            Self::En => include_str!("../../locales/en.json"),
            Self::Id => include_str!("../../locales/id.json"),
            Self::Zh => include_str!("../../locales/zh.json"),
        }
    }
}

static EN_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
static ID_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
static ZH_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

fn map_for(lang: Lang) -> &'static HashMap<String, String> {
    let cell = match lang {
        Lang::En => &EN_MAP,
        Lang::Id => &ID_MAP,
        Lang::Zh => &ZH_MAP,
    };
    cell.get_or_init(|| {
        // serde_json::from_str returns a generic Value first, then we
        // pull only the (k, String) pairs and drop reserved keys
        // (anything starting with "_") so translator-facing comments
        // don't pollute the lookup.
        match serde_json::from_str::<HashMap<String, serde_json::Value>>(lang.raw()) {
            Ok(raw) => raw
                .into_iter()
                .filter(|(k, _)| !k.starts_with('_'))
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                .collect(),
            Err(_) => HashMap::new(),
        }
    })
}

/// Lookup a translation. Falls back to the key itself on miss so a
/// missing translation surfaces as the key, never as a blank cell.
pub fn t(lang: Lang, key: &str) -> String {
    map_for(lang)
        .get(key)
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// Provide an `RwSignal<Lang>` via context. Call once at the App
/// boundary; consumers grab it with `use_lang()`.
pub fn provide_lang_signal() {
    let initial = read_persisted_lang();
    let signal = RwSignal::new(initial);
    provide_context(signal);
}

pub fn use_lang() -> RwSignal<Lang> {
    use_context::<RwSignal<Lang>>().expect("Lang RwSignal context not provided")
}

#[cfg(target_arch = "wasm32")]
fn read_persisted_lang() -> Lang {
    let Some(win) = web_sys::window() else {
        return Lang::En;
    };
    win.local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item("sentrix-lang").ok().flatten())
        .map(|s| Lang::from_code(&s))
        .unwrap_or(Lang::En)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_persisted_lang() -> Lang {
    Lang::En
}

#[cfg(target_arch = "wasm32")]
pub fn persist_lang(lang: Lang) {
    let Some(win) = web_sys::window() else { return };
    if let Ok(Some(storage)) = win.local_storage() {
        let _ = storage.set_item("sentrix-lang", lang.code());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn persist_lang(_lang: Lang) {}
