# Cosync

Fast, private, zero-cloud LAN sync — clipboard, files, and notifications between your devices, instant and local.

## Run & Operate

- `pnpm --filter @workspace/cosync run dev` — run the landing page (port 25788, artfact `artifacts/web`)
- `cd apps/desktop && npm install && npm run tauri dev` — run the desktop app in dev mode
- `cargo build -p cosync-core` — build the Rust library
- `cargo test -p cosync-core` — test the Rust library
- `pnpm run typecheck` — full TypeScript typecheck across all packages

## Stack

| Layer | Technology |
|---|---|
| Sync engine (Rust) | `crates/core/` — QUIC (quinn), mDNS-SD, Ed25519 identity, SQLite |
| Desktop shell | Tauri v2 (Rust), in `apps/desktop/src-tauri/` |
| Desktop UI | React 19, Vite 6, Tailwind CSS v4, in `apps/desktop/src/` |
| Landing page | React 19, Vite, Tailwind CSS v4, in `artifacts/web/` |
| Wire format | Protocol Buffers (prost-build) via `proto/cosync.proto` |

## Where things live

- **Rust crate**: `crates/core/` — all sync logic (transport, discovery, session, storage, pairing, HLC)
- **Desktop app**: `apps/desktop/` — React frontend + Tauri v2 shell
- **Desktop Tauri commands**: `apps/desktop/src-tauri/src/commands.rs` — 13 IPC commands
- **Landing page**: `artifacts/web/` — marketing site at `/`
- **Proto definition**: `proto/cosync.proto` — compiled by prost-build in build.rs
- **App icons source**: `assets/icons/logo.svg`
- **Documentation**: `docs/` — architecture.md, getting-started.md, protocol.md

## Architecture decisions

- **No cloud.** Every connection is direct peer-to-peer over LAN. No relay, no account, no server.
- **Trust via pairing.** Each device generates an Ed25519 keypair and self-signs a TLS certificate. Pairing stores the peer's fingerprint; all subsequent connections require a pinned match.
- **HLC for ordering.** Hybrid Logical Clocks provide a total causal order across devices without wall-clock synchronisation. Envelopes with >30s clock skew are dropped.
- **Prost-build for protos.** `crates/core/build.rs` navigates up to `proto/` to compile `cosync.proto` at build time.

## User preferences

_Populate as you build — explicit user instructions worth remembering across sessions._

## Gotchas

- `notify-rust ≥ 4.18` requires rustc ≥ 1.89. Pin to 4.11.3 if building with older toolchain: `cargo update notify-rust --precise 4.11.3`
- Tauri v2 does not accept `app.title` in `tauri.conf.json` — the title goes per-window only.
- Tauri requires RGBA PNGs (color type 6) for icons. RGB-only PNGs fail the build.

## Pointers

- See `pnpm-workspace` skill for monorepo structure and TypeScript setup.
- See `artifacts` skill for artifact registration and management.
