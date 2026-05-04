#!/usr/bin/env bash
# Sentrix Explorer V2 — release deployment.
#
# Builds both networks (one binary per network, since SENTRIX_NETWORK is
# compile-time baked) and ships them to the production host. Run from
# the build machine; never from a validator host.
#
# Pre-flight on the build machine:
#   - cargo, cargo-leptos, rustup target wasm32-unknown-unknown
#   - protoc ≥ 3.15
#
# Pre-flight on the production host:
#   - /etc/systemd/system/sentrix-explorer@.service installed
#   - /var/www/sentrix-explorer/{mainnet,testnet}/ exist + writable
#   - caddy block from deploy/Caddyfile merged into /etc/caddy/Caddyfile

set -euo pipefail

DEST_HOST="${DEST_HOST:-217.15.163.71}"
DEST_USER="${DEST_USER:-root}"
DEST_ROOT="${DEST_ROOT:-/var/www/sentrix-explorer}"

build_for() {
    local network="$1"
    echo "── building $network bundle ─────────────────────────"
    SENTRIX_NETWORK="$network" cargo leptos build --release
    # cargo-leptos drops:
    #   target/release/sentrix-explorer-v2          (axum SSR binary)
    #   target/site/                                 (HTML + /pkg WASM + assets)
    # Stage them into a per-network directory before rsync so two
    # release artifacts don't clobber each other in target/.
    local stage="target/dist/$network"
    rm -rf "$stage"
    mkdir -p "$stage"
    cp target/release/sentrix-explorer-v2 "$stage/"
    cp -r target/site "$stage/"
}

ship() {
    local network="$1"
    local stage="target/dist/$network"
    local dest="$DEST_ROOT/$network"
    echo "── shipping $network → $DEST_USER@$DEST_HOST:$dest ──"
    ssh "$DEST_USER@$DEST_HOST" "mkdir -p $dest"
    rsync -avz --delete "$stage/" "$DEST_USER@$DEST_HOST:$dest/"
}

restart() {
    local network="$1"
    echo "── restarting sentrix-explorer@$network ─────────────"
    ssh "$DEST_USER@$DEST_HOST" "systemctl restart sentrix-explorer@$network"
}

main() {
    for net in mainnet testnet; do
        build_for "$net"
        ship "$net"
        restart "$net"
    done
    echo
    echo "✓ deployment complete"
    echo "  mainnet → https://scan.sentriscloud.com"
    echo "  testnet → https://scan-testnet.sentriscloud.com"
}

main "$@"
