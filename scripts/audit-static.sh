#!/usr/bin/env bash
# Static-analysis sweep for the V2 (Leptos / Rust) explorer codebase.
# Mirror of the V1 audit-static.sh philosophy: a small set of grep
# rules that catch the bug classes we've actually shipped here.
#
# Run from repo root:  bash scripts/audit-static.sh
# Exit code = number of hard errors. CI can `set -e` on it.
#
# Add new rules at the bottom. Keep them grep-shaped so they stay
# maintainable — heavyweight Rust AST tools belong in clippy, not here.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Skip generated + vendor noise.
EXCLUDE='--exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git --exclude=Cargo.lock --exclude=*.lock'
SRC='src'

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
ylw()   { printf '\033[33m%s\033[0m\n' "$*"; }

ISSUES=0

section() {
  printf '\n──────  %s  ──────\n' "$1"
}

# Hard-error rule: print red lines, count toward exit code.
hard_rule() {
  local title="$1" pattern="$2" target="${3-$SRC}" exclude_extra="${4-}"
  section "$title"
  local hits
  # shellcheck disable=SC2086
  hits=$(grep -rnE $EXCLUDE "$pattern" $target 2>/dev/null || true)
  if [ -n "$exclude_extra" ]; then
    hits=$(echo "$hits" | grep -vE "$exclude_extra" || true)
  fi
  # Drop comment-only lines so commentary describing a bug isn't flagged
  # as the bug itself (matches V1 convention).
  hits=$(echo "$hits" | grep -vE '^[^:]+:[0-9]+:\s*//' || true)
  if [ -z "$hits" ]; then
    green "  clean"
    return 0
  fi
  echo "$hits" | while IFS= read -r line; do
    red "  ✗ $line"
  done
  local count
  count=$(printf '%s\n' "$hits" | wc -l)
  ISSUES=$((ISSUES + count))
}

# Soft rule: yellow warnings, no exit-code contribution.
soft_rule() {
  local title="$1" pattern="$2" target="${3-$SRC}" exclude_extra="${4-}"
  section "$title"
  local hits
  # shellcheck disable=SC2086
  hits=$(grep -rnE $EXCLUDE "$pattern" $target 2>/dev/null || true)
  if [ -n "$exclude_extra" ]; then
    hits=$(echo "$hits" | grep -vE "$exclude_extra" || true)
  fi
  hits=$(echo "$hits" | grep -vE '^[^:]+:[0-9]+:\s*//' || true)
  if [ -z "$hits" ]; then
    green "  clean"
    return 0
  fi
  local total
  total=$(echo "$hits" | wc -l)
  echo "$hits" | head -15 | while IFS= read -r line; do
    ylw "  ? $line"
  done
  if [ "$total" -gt 15 ]; then
    ylw "  ... ($((total - 15)) more)"
  fi
}

# ── Rule 1: hardcoded chain naming literals in network-aware paths ──
# Canonical: "Sentrix Chain" (mainnet) / "Sentrix Testnet" (testnet) /
# "SRX" symbol. Anywhere they appear inline in CONTENT/data shown for
# the *current* network is drift risk — should route through
# `config::Network::display()`. Surface-identity uses (footer brand
# title, navbar aria-label, OG meta) are intentionally site-fixed
# since the brand identity = "Sentrix Chain" regardless of the network
# this build serves.
soft_rule \
  "hardcoded chain names (verify site-identity vs network-aware)" \
  '"(Sentrix Chain|Sentrix Testnet|Sentrix Mainnet)"' \
  "$SRC" \
  '/(config\.rs|labels\.rs|state/network\.rs|context/network\.rs|i18n/)'

# ── Rule 2: hardcoded gRPC endpoints outside config ──
hard_rule \
  "hardcoded grpc endpoints (must live in config.rs)" \
  '"https?://grpc(-testnet)?\.sentrixchain\.com' \
  "$SRC" \
  '/config\.rs:'

# ── Rule 3: hardcoded scan domains outside config + footer/nav ──
# footer.rs + navbar.rs intentionally surface the user-facing URL.
soft_rule \
  "hardcoded scan domains (verify intentional surface text)" \
  '"https?://scan(-testnet)?\.sentriscloud\.com' \
  "$SRC" \
  '/(config\.rs|footer\.rs|navbar\.rs|labels\.rs|i18n/)'

# ── Rule 4: explorer URL composed with `?network=` BEFORE path ──
# Same V1 bug class — URL.set_search before path swallows the path
# into the query string.
hard_rule \
  "explorer URL with ?network= before path (broken composition)" \
  '"https?://scan[^"]*\?network=[^"]*/(tx|address|block)' \
  "$SRC"

# ── Rule 5: bare .unwrap() / .expect() outside tests + build.rs ──
# Browser WASM unwraps panic into uncaught exceptions; user sees a
# blank screen. Use `?` + an error type, or `unwrap_or_default`.
# Whitelist: `*tests*`, integration paths, build.rs.
soft_rule \
  ".unwrap()/.expect() in app paths (review — WASM panics blank-screen)" \
  '\.(unwrap|expect)\(' \
  "$SRC" \
  '/(tests?|__tests__)/|tests\.rs:'

# ── Rule 6: todo!() / unimplemented!() / unreachable!() shipped ──
hard_rule \
  "todo!()/unimplemented!() macros in shipped code" \
  '\b(todo|unimplemented)!\(' \
  "$SRC"

# ── Rule 7: println!/eprintln!/dbg! left in code ──
# WASM in browser: println goes nowhere; in SSR it pollutes stdout.
# Whitelist main.rs — server entrypoint startup banner is conventional
# eprintln (axum + leptos serve scripts do this everywhere).
hard_rule \
  "println!/eprintln!/dbg! left in code (use leptos::logging or tracing)" \
  '\b(println|eprintln|dbg)!\(' \
  "$SRC" \
  '/main\.rs:'

# ── Rule 8: TODO / FIXME / XXX / HACK markers ──
soft_rule \
  "TODO/FIXME/XXX/HACK markers (deferred work)" \
  '\b(TODO|FIXME|XXX|HACK)\b' \
  "$SRC"

# ── Rule 9: JSON-RPC use OUTSIDE the EVM bridge layer ──
# V2 talks gRPC for chain-native reads (block/balance/validator via
# `sentrix.v1.Sentrix`) but JSON-RPC for EVM-shaped reads (eth_call
# etc — those don't have gRPC equivalents). The JSON-RPC client is
# intentionally hand-rolled in `api/evm.rs` to avoid pulling in
# alloy-providers (~400 KB gzipped WASM bloat).
#
# So: JSON-RPC inside `api/evm.rs` + URL plumbing in `config.rs` /
# `state/network.rs` is whitelisted. Anywhere else is a layering
# violation — should call through the EvmProvider trait.
hard_rule \
  "raw JSON-RPC use outside the EVM bridge (route through EvmProvider)" \
  '"https?://rpc(-testnet)?\.sentrixchain\.com|"eth_(call|getBalance|blockNumber|chainId|gasPrice|getCode|sendRawTransaction)"' \
  "$SRC" \
  '/(api/evm\.rs|config\.rs|state/network\.rs):'

printf '\n────────────────────────────────────────\n'
if [ "$ISSUES" -eq 0 ]; then
  green "Static audit (V2): no hard errors."
else
  red "Static audit (V2): $ISSUES hard error(s). Yellow lines are review-only."
fi
exit "$ISSUES"
