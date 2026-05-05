use leptos::prelude::*;

use crate::components::live_feed::LiveBlockFeed;
use crate::components::mempool::MempoolWatcher;
use crate::components::stats::StatsPanel;
use crate::components::stats_dashboard::StatsDashboard;
use crate::components::validator_activity::ValidatorActivity;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="space-y-6">
            // New 4-card network stats grid (Etherscan/Solscan-style).
            // Mock data behind a Resource → loading skeleton on first
            // paint; TODOs in stats_dashboard::fetch_chain_stats for
            // the real RPC composition.
            <StatsDashboard />

            // Existing real-time panel — TPS sparkline, finality,
            // gas tracker, total supply progress. Lives below the
            // hero stats grid as a deeper-data dashboard.
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
