//! Theme toggle — Obsidian (zinc dark) vs Solar (high-contrast light).
//!
//! Why class-on-html instead of a Leptos signal threaded through every
//! component:
//!   1. Pre-paint set by `apply_persisted_theme()` from the hydrate
//!      hook means there's no flash-of-unstyled-content while the
//!      framework boots.
//!   2. Class-driven CSS overrides (see `style/tailwind.css` `.solar`
//!      block) sidestep refactoring every existing zinc-* class.
//!
//! Persistence: `localStorage["sentrix-theme"] = "solar" | "obsidian"`.
//! Default = obsidian (matches the un-classed root html state).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Obsidian,
    Solar,
}

impl Theme {
    pub fn class(self) -> &'static str {
        match self {
            Self::Obsidian => "obsidian",
            Self::Solar => "solar",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Obsidian => Self::Solar,
            Self::Solar => Self::Obsidian,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Obsidian => "Obsidian",
            Self::Solar => "Solar",
        }
    }
    #[cfg(target_arch = "wasm32")]
    fn parse(s: &str) -> Self {
        match s {
            "solar" => Self::Solar,
            _ => Self::Obsidian,
        }
    }
}

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "sentrix-theme";

#[cfg(target_arch = "wasm32")]
pub fn apply_persisted_theme() {
    let win = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let stored = win
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
        .map(|v| Theme::parse(&v))
        .unwrap_or(Theme::Obsidian);

    set_html_class(stored);
}

#[cfg(target_arch = "wasm32")]
pub fn toggle() -> Theme {
    let win = match web_sys::window() {
        Some(w) => w,
        None => return Theme::Obsidian,
    };
    let storage = win.local_storage().ok().flatten();

    let current = storage
        .as_ref()
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
        .map(|v| Theme::parse(&v))
        .unwrap_or(Theme::Obsidian);

    let next = current.next();
    if let Some(s) = storage {
        let _ = s.set_item(STORAGE_KEY, next.class());
    }
    set_html_class(next);
    next
}

#[cfg(target_arch = "wasm32")]
fn set_html_class(theme: Theme) {
    let Some(win) = web_sys::window() else { return };
    let Some(doc) = win.document() else { return };
    let Some(html) = doc.document_element() else {
        return;
    };
    let list = html.class_list();
    // Solar mode is opt-in; Obsidian is "no class" so the default
    // styles serve as the dark canvas without a class round-trip.
    let _ = list.remove_1("solar");
    if theme == Theme::Solar {
        let _ = list.add_1("solar");
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn apply_persisted_theme() {
    // SSR pre-render uses the default Obsidian theme — the hydrate
    // hook flips to the persisted choice once localStorage is reachable.
}

#[cfg(not(target_arch = "wasm32"))]
pub fn toggle() -> Theme {
    Theme::Obsidian
}
