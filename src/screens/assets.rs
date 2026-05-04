use leptos::prelude::*;

use crate::components::rwa::AssetList;

#[component]
pub fn AssetsScreen() -> impl IntoView {
    // Empty list ships today; when the partnership registry lands,
    // swap to a `Resource` that pulls from the Sentrix asset feed.
    view! {
        <div class="space-y-6">
            <AssetList />
        </div>
    }
}
