# Sentrix Explorer V2 — Obsidian Engine

[![CI](https://github.com/Sentriscloud/sentrix-explorer-v2/actions/workflows/ci.yml/badge.svg)](https://github.com/Sentriscloud/sentrix-explorer-v2/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/Sentriscloud/sentrix-explorer-v2)](LICENSE)
[![Latest release](https://img.shields.io/github/v/release/Sentriscloud/sentrix-explorer-v2?include_prereleases&sort=semver)](https://github.com/Sentriscloud/sentrix-explorer-v2/releases/latest)


Full-Rust block explorer for Sentrix Chain. Browser-side WASM bundle talks
to the chain over gRPC-Web (`grpc.sentrixchain.com (mainnet) / grpc-testnet.sentrixchain.com (testnet)`); zero JSON-RPC, zero
JavaScript glue.

**Live:**
- Mainnet: https://scan.sentriscloud.com
- Testnet: https://scan-testnet.sentriscloud.com

> **Two explorers coexist by design.** This is the **WASM V2 Obsidian** — full-Rust + tonic-web, near-native parse cost, signal-driven UI. For the **Next.js V1** alternative — feature-rich (validator pages, leaderboard, EIP-3091 deeplinks, contract verification panel, multi-locale i18n) — see [`Sentriscloud/frontend/apps/scan`](https://github.com/Sentriscloud/frontend/tree/main/apps/scan) at `scan.sentrixchain.com` / `scan-testnet.sentrixchain.com`. Pick whichever fits the workflow; neither replaces the other.

The gRPC-Web wrapper used here has been extracted into a standalone crate — see [`Sentriscloud/sentrix-grpc-wasm`](https://github.com/Sentriscloud/sentrix-grpc-wasm) — so other browser dApps (Yew, plain wasm-bindgen) can reuse it without re-implementing the `tonic-web-wasm-client` glue.

## Architecture

```
proto/sentrix.proto       single source of truth (mirrored from sentrix/crates/sentrix-grpc)
   │
   ▼  build.rs / tonic-build
src/grpc/pb               prost types + tonic client stubs
   │
   ▼
src/grpc/client.rs        SentrixGrpcClient — wraps tonic-web-wasm-client transport
   │
   ▼
src/components/live_feed  signal-driven LiveBlockFeed (stream → poll fallback)
   │
   ▼
src/screens/dashboard     route view
```

## Why this beats Etherscan-class explorers

| Surface         | React/Next stack                  | Obsidian Engine                  |
|-----------------|-----------------------------------|----------------------------------|
| Wire format     | JSON-RPC (text, verbose)          | gRPC-Web (binary, ~3-5× smaller) |
| Render path     | Virtual DOM diff                  | Fine-grained signals             |
| Parse cost      | JS interpreter                    | WASM near-native                 |
| Type safety     | Runtime                           | Compile-time (proto ↔ struct)    |

If the chain proto changes, this crate fails to compile — the UI literally
cannot drift from the wire contract.

## Build & run

### One-time

```sh
cargo install cargo-leptos --locked
rustup target add wasm32-unknown-unknown
```

### Dev

```sh
cargo leptos watch       # SSR on :3000, HMR on :3001
```

### Release

```sh
cargo leptos build --release
# binary:  target/release/sentrix-explorer-v2
# assets:  target/site/
```

## Production URLs

- Mainnet · https://scan.sentriscloud.com
- Testnet · https://scan-testnet.sentriscloud.com

Deploy artifacts (Caddyfile, systemd unit, deploy.sh) live in `deploy/`.

## Streaming status

Chain `gRPC v0.2` (active 2026-05-04) ships only `GetBlock` + `GetBalance`.
`StreamEvents` returns `Status::unimplemented`; `LiveBlockFeed` falls back
to a 2 s `GetBlock(latest)` poll. The streaming arm activates automatically
when chain `v0.3` ships — no client change needed.
