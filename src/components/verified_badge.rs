//! VerifiedBadge — small chip used on canonical/verified entities.
//!
//! Three states aligned with `VerificationStatus` (RWA module reuses
//! the same shape via a different enum). Keeping the chip in its own
//! component means the "✓ Verified" semantics is one definition the
//! whole UI shares.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifiedStatus {
    Verified,
    Pending,
    Unverified,
}

impl VerifiedStatus {
    fn classes(self) -> &'static str {
        match self {
            Self::Verified => "border-emerald-500/30 bg-emerald-500/10 text-emerald-300",
            Self::Pending => "border-amber-500/30 bg-amber-500/10 text-amber-300",
            Self::Unverified => "border-zinc-700 bg-zinc-800/40 text-zinc-400",
        }
    }
    fn glyph(self) -> &'static str {
        match self {
            Self::Verified => "✓ Verified",
            Self::Pending => "… Pending",
            Self::Unverified => "Unverified",
        }
    }
}

#[component]
pub fn VerifiedBadge(status: VerifiedStatus) -> impl IntoView {
    let class = format!(
        "inline-flex items-center gap-1 rounded-md border px-2 py-0.5 font-mono text-[10px] uppercase tracking-wider {}",
        status.classes()
    );
    view! { <span class=class>{status.glyph()}</span> }
}
