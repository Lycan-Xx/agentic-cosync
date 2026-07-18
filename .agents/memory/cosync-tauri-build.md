---
name: Cosync Tauri v2 build setup
description: What was needed to build the Cosync Tauri v2 desktop app from .migration-backup/cosync on Linux/Replit
---

## Summary
Building `.migration-backup/cosync/apps/desktop` (Tauri v2 + Rust + React/Vite) on Replit Linux.

## Tools installed
- `installProgrammingLanguage({ language: "rust-stable" })` — Rust 1.88.0
- `installSystemDependencies` with: `webkitgtk_4_1`, `pkg-config`, `openssl`, `gtk3`, `librsvg`, `glib`, `pango`, `at-spi2-atk`, `libsoup_3`, `gdk-pixbuf`, `protobuf`, `xorg.libX11`, `xorg.libXext`, `libayatana-appindicator`
- `npm install` in `apps/desktop/` for JS deps

## Code fixes required
1. **`tauri.conf.json`**: `app.title` is not valid in Tauri v2 — title goes only in `app.windows[].title`.
2. **Tailwind v4 + Vite**: Desktop app uses `@import "tailwindcss"` but had no Tailwind plugin. Fix: `npm install tailwindcss @tailwindcss/vite`, add `tailwindcss()` to `vite.config.ts` plugins, add empty `postcss.config.mjs` to shadow the parent directory's config.
3. **`notify-rust` MSRV**: `notify-rust@4.18.0` requires rustc 1.89.0 but stable is 1.88.0. Fix: `cargo update notify-rust --precise 4.11.3` from the workspace root.
4. **`ed25519_dalek::pkcs8`**: Not a direct dep of `cosync-desktop`. Fix: add `ed25519-dalek = { workspace = true }` to `apps/desktop/src-tauri/Cargo.toml`. Also change `use ed25519_dalek::pkcs8::EncodePrivateKey;` to `use ed25519_dalek::pkcs8::EncodePrivateKey as _;` (trait used but name not needed).
5. **`start_server` missing `.await`**: `SessionManager::start_server` is async; the call site in `commands.rs` was missing `.await`.
6. **`DiscoveryService`, `Storage` not re-exported**: `cosync-core/src/lib.rs` lacked `pub use discovery::DiscoveryService;` and `pub use storage::Storage;`. Added both.
7. **Missing icons**: `src-tauri/icons/` directory didn't exist. Tauri requires RGBA PNG icons at `32x32.png`, `128x128.png`, `128x128@2x.png`, plus `icon.icns` and `icon.ico`. Generated via Node.js. **Must be RGBA color type (PNG color type 6), not RGB.**
8. **`event_rx` mutability**: `let event_rx` needed `let mut event_rx` for `.recv().await` in the event loop.
9. **Type inference on `DiscoveryService` methods**: `if let Some(ref x) = *guard` caused inference failures; change to `guard.as_ref()` pattern.

**Why:** These are all integration gaps between the desktop Tauri shell and the cosync-core library that built fine in isolation but had import/API surface mismatches when combined.

## Build command
```sh
cd .migration-backup/cosync/apps/desktop
npm run tauri build -- --bundles deb
```
Output: `target/release/bundle/deb/Cosync_0.1.0_amd64.deb`
Compile time: ~1m 25s (incremental after first full build).
