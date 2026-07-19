# Cosync

> Fast, private, zero-cloud LAN sync — clipboard, files, and notifications between your devices, instant and local.

Cosync lets devices on the same local network share clipboard content, transfer files, and mirror notifications without routing anything through the internet or a third party. Everything runs peer-to-peer over QUIC on your LAN.

---

## Project Structure

```
cosync/
├── crates/
│   └── core/              # cosync-core — the Rust sync engine
│       └── src/           # QUIC transport, mDNS, SQLite, HLC, protobuf
├── apps/
│   ├── desktop/           # Cosync Desktop (Tauri v2 + React 19)
│   │   ├── src/           # React UI — hooks, components, pages
│   │   └── src-tauri/     # Tauri shell — IPC commands, plugins, Rust glue
│   └── web/               # cosync.app landing page (React 19 + Vite)
├── proto/
│   └── cosync.proto       # Protobuf wire format (compiled by prost-build)
├── docs/
│   ├── architecture.md    # System design and data flow
│   ├── getting-started.md # Build & run for each platform
│   └── protocol.md        # Wire protocol reference
└── assets/
    └── icons/             # Source app icons (SVG → Tauri icon set)
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Network transport | QUIC — [quinn](https://github.com/quinn-rs/quinn) |
| Device discovery | mDNS-SD — [mdns-sd](https://github.com/keepsimple1/mdns-sd) |
| Crypto identity | Ed25519 (ed25519-dalek) + self-signed TLS cert (rcgen) |
| Storage | SQLite via rusqlite (bundled, no system lib needed) |
| Wire format | Protocol Buffers compiled by prost-build |
| Ordering | Hybrid Logical Clocks (HLC) — causal ordering across devices |
| Desktop shell | Tauri v2 |
| Desktop UI | React 19 + Vite 6 + Tailwind CSS v4 |
| Landing page | React 19 + Vite + Tailwind CSS v4 |

---

## Architecture Overview

```
┌──────────────────────────┐       LAN / mDNS        ┌──────────────────────────┐
│  Device A                │◄───────────────────────►│  Device B                │
│                          │                          │                          │
│  ┌────────────────────┐  │    QUIC (mutual TLS)    │  ┌────────────────────┐  │
│  │   cosync-core      │◄─┼─────────────────────────┼─►│   cosync-core      │  │
│  │                    │  │                          │  │                    │  │
│  │  ┌──────────────┐  │  │                          │  │  ┌──────────────┐  │  │
│  │  │ SessionMgr   │  │  │                          │  │  │ SessionMgr   │  │  │
│  │  │ Discovery    │  │  │                          │  │  │ Discovery    │  │  │
│  │  │ Storage      │  │  │                          │  │  │ Storage      │  │  │
│  │  │ FileTransfer │  │  │                          │  │  │ FileTransfer │  │  │
│  │  └──────────────┘  │  │                          │  │  └──────────────┘  │  │
│  └─────────┬──────────┘  │                          │  └─────────┬──────────┘  │
│            │ Tauri IPC   │                          │            │ Tauri IPC   │
│  ┌─────────▼──────────┐  │                          │  ┌─────────▼──────────┐  │
│  │  React UI           │  │                          │  │  React UI           │  │
│  └────────────────────┘  │                          │  └────────────────────┘  │
└──────────────────────────┘                          └──────────────────────────┘
```

See [docs/architecture.md](docs/architecture.md) for a full breakdown.

---

## Quick Start

### Prerequisites

- **Rust** ≥ 1.88 — install via [rustup](https://rustup.rs)
- **Node.js** ≥ 20 + **npm** (for the desktop frontend)
- **protoc** — Protocol Buffers compiler (needed by `prost-build` at compile time)
- **Tauri system libs** (Linux only) — see [docs/getting-started.md](docs/getting-started.md)

### Build the Desktop App

```bash
cd apps/desktop
npm install
npm run tauri build -- --bundles deb    # Linux .deb
npm run tauri build -- --bundles dmg    # macOS .dmg
npm run tauri build -- --bundles msi    # Windows .msi
```

The built binary lands in `target/release/cosync-desktop`.  
Bundles are in `target/release/bundle/<format>/`.

### Run the Landing Page

```bash
cd apps/web
pnpm install
pnpm dev
```

### Build cosync-core only (Rust library)

```bash
cargo build -p cosync-core
```

---

## Milestones

| # | Feature | Status |
|---|---|---|
| M1 | Ed25519 identity + rcgen TLS cert | ✅ Done |
| M2 | QUIC transport (quinn, mutual TLS) | ✅ Done |
| M3 | mDNS-SD discovery | ✅ Done |
| M4 | Session manager + device pairing | ✅ Done |
| M5 | Clipboard sync + local monitor | ✅ Done |
| M6 | File transfer (chunked QUIC streams) | ✅ Done |
| M7 | Notification mirroring | ✅ Done |
| M8 | Desktop app (Tauri v2 + React) | ✅ Done |
| M9 | Mobile app (React Native + UniFFI) | 🔲 Planned |
| M10 | E2E test suite + packaging polish | 🔲 Deferred |

---

## IPC Command Reference

The Tauri backend exposes these commands to the React frontend via `invoke()`:

| Command | Description |
|---|---|
| `get_device_info` | Returns device name + SHA-256 fingerprint |
| `get_device_fingerprint` | Returns just the fingerprint |
| `get_connection_state` | Returns current `ConnectionState` as a string |
| `start_discovery` | Starts mDNS browse + QUIC server + event forwarding |
| `stop_discovery` | Shuts down the session and discovery service |
| `pair_with_device(ip, port, fp)` | Initiates QUIC pairing with a discovered peer |
| `unpair_device(device_id)` | Removes a device from the paired-devices database |
| `get_paired_devices` | Lists all previously paired devices from SQLite |
| `get_clipboard_history` | Returns the last 100 clipboard entries |
| `send_clipboard(content)` | Broadcasts clipboard text to all connected peers |
| `clear_clipboard_history` | Deletes all clipboard history from SQLite |
| `send_file(file_path)` | Chunks and streams a file to all connected peers |
| `open_file_in_folder(path)` | Opens the system file manager at the file's location |

---

## License

MIT
