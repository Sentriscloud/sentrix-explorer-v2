//! Skeleton — neutral loading placeholder.
//!
//! Use anywhere you'd otherwise show a spinner. The shimmering bar
//! reads as "we're still working" without committing to a layout
//! that might shift when real data lands.

use leptos::prelude::*;

#[component]
pub fn Skeleton(
    /// Tailwind width / height utility classes (e.g. `"h-4 w-32"`).
    #[prop(into)]
    class: String,
) -> impl IntoView {
    let class = format!("skeleton-shimmer rounded-md bg-zinc-800/60 {class}");
    view! { <div class=class /> }
}

#[component]
pub fn SkeletonRow() -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 rounded-xl border border-zinc-800/30 bg-zinc-900/40 p-3">
            <Skeleton class="h-10 w-10" />
            <div class="flex-1 space-y-2">
                <Skeleton class="h-3 w-24" />
                <Skeleton class="h-3 w-40" />
            </div>
            <Skeleton class="h-3 w-16" />
        </div>
    }
}
