//! Canonical contract registry — operator-curated list of verified
//! contracts (token canonicals, governance, treasury, etc.).
//!
//! Empty by default. Populated either by:
//!   - a future static registry file shipped under `locales/` /
//!     `assets/`, or
//!   - a server endpoint once the explorer has a backend round-trip.
//!
//! Layout-only today.

use leptos::prelude::*;

use crate::components::verified_badge::VerifiedStatus;

#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalContract {
    pub name: String,
    /// 0x-prefixed lowercase EVM address.
    pub address: String,
    /// Optional one-line role label ("Token canonical", "Treasury",
    /// "Governance", etc.) for UI.
    pub role: String,
    pub status: VerifiedStatus,
}

#[derive(Clone, Copy)]
pub struct CanonicalRegistryState {
    pub contracts: ReadSignal<Vec<CanonicalContract>>,
}

pub fn provide_canonical_registry() {
    // Empty list ships today. When the curated registry source lands
    // (static JSON or server endpoint), swap this to a `Resource`.
    let (contracts, _set) = signal(Vec::<CanonicalContract>::new());
    provide_context(CanonicalRegistryState { contracts });
}
