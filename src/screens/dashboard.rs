use leptos::prelude::*;

use crate::components::live_feed::LiveBlockFeed;
use crate::components::mempool::MempoolWatcher;
use crate::components::stats_dashboard::StatsDashboard;
use crate::components::validator_activity::ValidatorActivity;

#[component]
pub fn Dashboard() -> impl IntoView {
    // Trimmed from 12 cards → 4 (audit feedback). The deeper StatsPanel
    // had three duplicates (Tip Height = Latest Block, both TPS variants
    // were the same data, "Avg Finality 0.00s" had no real data) plus
    // empty TPS / gas charts that read as broken. Keep the hero +
    // canonical live surfaces — supply progress lives in the hero card
    // alongside Latest Block now.
    view! {
        <div class="space-y-6">
            <StatsDashboard />

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
