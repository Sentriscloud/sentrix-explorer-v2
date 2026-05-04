use leptos::prelude::*;

use crate::components::live_feed::LiveBlockFeed;
use crate::components::mempool::MempoolWatcher;
use crate::components::stats::StatsPanel;
use crate::components::validator_activity::ValidatorActivity;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <StatsPanel />
            <div class="grid gap-6 lg:grid-cols-5">
                <div class="lg:col-span-3">
                    <LiveBlockFeed />
                </div>
                <div class="space-y-6 lg:col-span-2">
                    <MempoolWatcher />
                    <ValidatorActivity />
                </div>
            </div>
        </div>
    }
}
