# Cosync — Architecture

## Overview

Cosync synchronises clipboard content, files, and notifications between devices on the same LAN using a fully peer-to-peer design. No cloud server, no relay, no account required.

```
┌──────────────────────────┐       LAN / mDNS        ┌──────────────────────────┐
│  Device A                │◄───────────────────────►│  Device B                │
│                          │                          │                          │
│  ┌────────────────────┐  │    QUIC (mutual TLS)    │  ┌────────────────────┐  │
│  │   cosync-core      │◄─┼─────────────────────────┼─►│   cosync-core      │  │
│  └────────┬───────────┘  │                          │  └────────┬───────────┘  │
│           │ Tauri IPC    │                          │           │ Tauri IPC    │
│  ┌────────▼───────────┐  │                          │  ┌────────▼───────────┐  │
│  │  React UI          │  │                          │  │  React UI          │  │
│  └────────────────────┘  │                          │  └────────────────────┘  │
└──────────────────────────┘                          └──────────────────────────┘
```

---

## Layers

### 1 — `cosync-core` (`crates/core/`)

The entire sync engine lives here as a pure Rust library. It has no knowledge of Tauri or any UI framework, so it can be reused for mobile (UniFFI), CLI tools, or tests.

| Module | Responsibility |
|---|---|
| `identity` | Generate/load an Ed25519 keypair and self-sign a TLS certificate with rcgen. The certificate's public-key SHA-256 is the device fingerprint. |
| `transport` | Build QUIC client/server endpoints with mutual TLS. Only pinned fingerprints are accepted — this is the trust model. |
| `discovery` | Advertise and browse mDNS-SD (`_cosync._udp.local`). Converts `ServiceEvent`s into typed `DiscoveredPeer` structs. |
| `session` | Manage QUIC connections, accept incoming streams, dispatch envelopes, forward `SessionEvent`s. Central coordination point. |
| `pairing` | Exchange `PairingRequest`/`PairingAck` envelopes over a direct QUIC stream. Stores paired device fingerprints in SQLite. |
| `storage` | SQLite via rusqlite (bundled). Tables: `devices` (paired), `clipboard_history`, `file_transfers`. |
| `hlc` | Hybrid Logical Clock. Assigns causal timestamps to every envelope so out-of-order delivery can be detected and ignored. |
| `envelope` | Builder functions for each message type (clipboard, file meta/chunk/ack, notification, pairing, heartbeat). |
| `wire` | prost-generated Rust types from `proto/cosync.proto`. Do not edit by hand. |
| `file_transfer` | Chunk a file into fixed-size QUIC payloads; reassemble on the receiving side; verify SHA-256. |
| `clipboard` | (Planned) OS clipboard monitor via arboard — watches for local changes and fires events. |
| `notification` | Stub for Android notification mirroring via a companion mobile app. |

### 2 — `cosync-desktop` (`apps/desktop/src-tauri/`)

Thin Tauri v2 shell that exposes `cosync-core` capabilities to the React frontend via `invoke()`.

- **`state.rs`** — `CosyncState`: shared application state wrapped in `Arc<Mutex<_>>` and managed by Tauri's state system.
- **`commands.rs`** — 13 `#[tauri::command]` functions, one per IPC call. They lock state, call into cosync-core, and return serialisable results.
- **`lib.rs`** — Tauri builder: registers plugins (shell, dialog, fs, notification), sets up tracing to a log file, and registers all commands.

### 3 — Desktop UI (`apps/desktop/src/`)

React 19 + Vite 6 + Tailwind CSS v4.

- **`useCosync`** hook — wraps all `invoke()` calls and `listen()` event subscriptions; surfaces typed state to components.
- Three main tabs: **Devices** (discovery + pairing), **Clipboard** (history, send), **Files** (send, progress, open in folder).
- Events arrive on the `cosync://event` Tauri channel as typed `FrontendEvent` variants.

### 4 — Landing Page (`artifacts/web/`)

React 19 + Vite + Tailwind. Standalone marketing/download page served as a static Replit web artifact. No shared code with the desktop UI.

---

## Trust Model

Every device generates a unique Ed25519 keypair on first launch and self-signs a TLS certificate. The public-key SHA-256 is the device's **fingerprint** — human-verifiable, displayed in the UI.

Pairing: Device A connects to Device B over QUIC and sends a `PairingRequest` containing its fingerprint and device name. Device B displays the request; if the user confirms, it sends a `PairingAck` and both sides add each other's fingerprint to a `PinnedCertVerifier`. Subsequent connections are rejected unless the remote fingerprint is in the pinned set.

There is no central authority, no CA, and no pre-shared key. Trust is established by explicit human approval of a fingerprint, similar to SSH known-hosts.

---

## Message Flow — Clipboard Sync

```
User copies text on Device A
        │
        ▼
  arboard watcher fires
        │
        ▼
  HLC.now() → timestamp
        │
        ▼
  envelope::clipboard_envelope(device_id, hlc_ts, content, "text/plain")
        │
        ▼
  SessionManager::broadcast(envelope)
        │  ┌──────────────── for each connected peer ────────────────────┐
        ▼  │                                                              │
  QUIC stream open → send_envelope → flush                               │
        │  └──────────────────────────────────────────────────────────────┘
        │
    Device B
        │
        ▼
  recv_envelope → deserialise Envelope
        │
        ▼
  HLC.receive(hlc_ts)  — detect + reject clock skew > 30s
        │
        ▼
  handle_envelope → SessionEvent::ClipboardReceived
        │
        ▼
  event_tx.send → Tauri emit("cosync://event")
        │
        ▼
  React listen() → setState → ClipboardPanel updates
```

---

## Data Persistence

SQLite (bundled rusqlite) is opened at app-data-dir on startup. Three tables:

| Table | Contents |
|---|---|
| `devices` | Paired device ID, name, fingerprint, last known IP, last seen timestamp |
| `clipboard_history` | Content bytes, MIME type, source device ID, HLC timestamp, wall time |
| `file_transfers` | Transfer ID, file name, size, SHA-256, chunk count, status |

The database path is resolved via Tauri's `path().app_data_dir()` so it follows OS conventions (`~/.local/share/cosync-desktop/` on Linux, `~/Library/Application Support/cosync-desktop/` on macOS).

---

## Build Graph

```
proto/cosync.proto
       │ prost-build (build.rs)
       ▼
crates/core/src/wire.rs   (generated, do not edit)
       │
       ▼
crates/core   ──────────────────────────────────────────┐
       │                                                 │
       ▼                                                 ▼
apps/desktop/src-tauri   (Rust, Tauri v2 shell)      (future: mobile UniFFI)
       │
       ├── apps/desktop/src/   (React, compiled by Vite inside Tauri build)
       │
       ▼
target/release/cosync-desktop   (binary)
target/release/bundle/          (deb / dmg / msi)
```
