//! Identicon — deterministic geometric avatar from a hex string.
//!
//! Pure-Rust SVG generator. No JS, no network, no PNG raster. Each cell
//! is a `<rect/>`, the whole avatar is ≤25 rects + 1 background, and
//! the SVG is static after first mount — Leptos keyed iteration in the
//! parent feed means only newly-prepended rows render fresh ones.
//!
//! ## Determinism
//!
//! The input is already a Keccak-derived hash (block.hash for the live
//! feed; works equally well for any 20/32-byte hex address). Slicing
//! fixed bytes from a Keccak digest gives a uniform 8-bit distribution,
//! so we don't need a separate hash crate — just `hex::decode`.
//!
//! Layout: 5×5 grid mirrored about the vertical centre. Columns 0/1/2
//! are unique (15 cells), columns 3/4 mirror 1/0. The result is
//! left-right symmetrical, which reads as "intentional / minimal" to
//! the eye while still encoding 15 bits of identity.
//!
//! Bit assignments from the parsed hash bytes:
//!     byte[0] low 3 bits  → background slot   (8-colour zinc palette)
//!     byte[1] low 3 bits  → foreground slot   (8-colour pop palette)
//!     bytes[2..=3]        → 15 cell on/off bits (1 bit unused)

use leptos::prelude::*;

/// Muted zinc/slate/stone backgrounds. All sit comfortably on the
/// `bg-zinc-900/40` row tile so the identicon doesn't pop louder than
/// the row text. Chosen for ≥ 4.5:1 contrast with foreground palette.
const BG_PALETTE: [&str; 8] = [
    "#27272a", // zinc-800
    "#1f2937", // gray-800
    "#1e293b", // slate-800
    "#292524", // stone-800
    "#3f3f46", // zinc-700
    "#374151", // gray-700
    "#334155", // slate-700
    "#44403c", // stone-700
];

/// Foreground "pops" — saturated tints kept at 400/500 weight so they
/// punch through the dark cell without going neon. Amber is first so
/// brand-default accents are slightly more common.
const FG_PALETTE: [&str; 8] = [
    "#f59e0b", // amber-500
    "#fb923c", // orange-400
    "#38bdf8", // sky-400
    "#2dd4bf", // teal-400
    "#34d399", // emerald-400
    "#a78bfa", // violet-400
    "#fb7185", // rose-400
    "#facc15", // yellow-400
];

/// Lenient hex parse — strip optional `0x`, take up to 4 bytes, pad with
/// zero on short input. Means a malformed hash still produces a stable
/// (empty/blank) identicon instead of panicking the WASM bundle.
fn first_four_bytes(hex: &str) -> [u8; 4] {
    let cleaned = hex.strip_prefix("0x").unwrap_or(hex);
    let bytes = cleaned.as_bytes();
    let mut out = [0u8; 4];
    for (i, slot) in out.iter_mut().enumerate() {
        let hi = bytes.get(i * 2).copied().unwrap_or(b'0');
        let lo = bytes.get(i * 2 + 1).copied().unwrap_or(b'0');
        let byte = match (hex_nibble(hi), hex_nibble(lo)) {
            (Some(h), Some(l)) => (h << 4) | l,
            _ => 0,
        };
        *slot = byte;
    }
    out
}

const fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Compute the (col, row) coordinates of every "on" cell, applying the
/// horizontal mirror. Output is at most 25 cells in viewBox units.
fn cells_from(bits: u16) -> Vec<(u32, u32)> {
    let mut out = Vec::with_capacity(25);
    for col in 0u32..3 {
        for row in 0u32..5 {
            let idx = col * 5 + row;
            if (bits >> idx) & 1 == 1 {
                out.push((col, row));
                if col < 2 {
                    out.push((4 - col, row));
                }
            }
        }
    }
    out
}

#[component]
pub fn Identicon(
    /// Hex string used to derive the avatar. Block hash, address, or
    /// any deterministic hex identifier — first 4 bytes are consumed.
    address_hex: String,
    /// Pixel size of the rendered SVG. Defaults to 32; the spec
    /// targets 24-32 for in-row use.
    #[prop(default = 32)]
    size: u8,
) -> impl IntoView {
    let bytes = first_four_bytes(&address_hex);
    let bg = BG_PALETTE[(bytes[0] & 0x07) as usize];
    let fg = FG_PALETTE[(bytes[1] & 0x07) as usize];
    let bits = bytes[2] as u16 | ((bytes[3] as u16) << 8);
    let cells = cells_from(bits);

    let size_str = size.to_string();

    view! {
        <svg
            width=size_str.clone()
            height=size_str
            viewBox="0 0 5 5"
            shape-rendering="crispEdges"
            class="rounded-lg"
            aria-hidden="true"
        >
            <rect width="5" height="5" fill=bg />
            {cells
                .into_iter()
                .map(|(x, y)| {
                    view! {
                        <rect
                            x=x.to_string()
                            y=y.to_string()
                            width="1"
                            height="1"
                            fill=fg
                        />
                    }
                })
                .collect_view()}
        </svg>
    }
}
