# Cosync — LAN Device Sync

**Clipboard, Files, Notifications — synced across your devices over your local network.**

Cosync is a peer-to-peer device synchronization system that works entirely over LAN (no cloud, no accounts). It uses QUIC for fast, multiplexed transport, mDNS for zero-config peer discovery, and pinned self-signed certificates for security. The desktop app is built with Tauri v2 (Rust backend + React frontend), keeping the binary under 10 MB.

## Architecture

```
+------------------+        mDNS         +------------------+
|  Desktop (Tauri) |<--- _cosync._udp ---|  Mobile (RN)     |
|  +-----------+   |                       |  +-----------+   |
|  | React UI  |   |       QUIC/TLS 1.3    |  | React UI  |   |
|  +-----+-----+   |<=====================>|  +-----+-----+   |
|        |         |                       |        |         |
|  +-----+-----+   |                       |  +-----+-----+   |
|  | Tauri CMD |   |                       |  | UniFFI    |   |
|  +-----+-----+   |                       |  | Kotlin NM |   |
|        |         |                       |        |         |
|  +-----+----------------------------------------------------+
|  |                  cosync-core (Rust)                      |
|  +----------------------------------------------------------+
+--------------------------------------------------------------+
```

## Key Technologies

| Component | Tech |
|---|---|
| Transport | QUIC via quinn + rustls (TLS 1.3) |
| Discovery | mDNS/DNS-SD (mdns-sd, `_cosync._udp.local`) |
| Security | Pinned self-signed certs (Ed25519 + X.509) |
| Ordering | Hybrid Logical Clock (HLC) for causal consistency |
| Storage | SQLite via rusqlite (WAL mode) |
| Wire Format | Protobuf via prost |
| Desktop Shell | Tauri v2 + React 19 + TypeScript + Vite 6 |
| Styling | Tailwind CSS v4 |
| Mobile Bridge | UniFFI → Kotlin → React Native native module (planned) |

## Desktop App (M8 — Tauri v2)

The desktop app lives in `apps/desktop/`. It follows the standard Tauri v2 project layout:

```
apps/desktop/
+-- package.json              # React/TS frontend deps
+-- vite.config.ts            # Vite dev server (port 1420)
+-- tsconfig.json
+-- index.html                # Vite entry point
+-- src/                      # React frontend
|   +-- main.tsx              # ReactDOM render
|   +-- App.tsx               # Root component — tabbed layout (Devices / Clipboard / Files)
|   +-- styles.css            # Tailwind v4 + dark theme
|   +-- hooks/
|   |   +-- useCosync.ts      # Core state hook — discovery, pairing, events, clipboard, files
|   +-- lib/
|   |   +-- commands.ts       # Typed Tauri IPC wrappers (12 commands)
|   +-- types/
|   |   +-- events.ts         # Shared Rust↔TS type definitions (FrontendEvent tagged union)
|   +-- components/
|       +-- ui.tsx            # StatusBadge, DeviceCard, PeerList, ErrorBanner
|       +-- ClipboardPanel.tsx # Clipboard send bar + history list + copy-to-clipboard
|       +-- FileTransferPanel.tsx # Drag-and-drop file picker + transfer progress + open-in-folder
+-- src-tauri/                # Rust backend (Tauri crate)
    +-- Cargo.toml            # Depends on cosync-core + tauri + plugins + hex
    +-- build.rs              # tauri_build::build()
    +-- tauri.conf.json       # App config, window, bundle settings
    +-- capabilities/
    |   +-- default.json      # Tauri v2 permission grants
    +-- src/
        +-- main.rs           # Entry point (windows_subsystem = "windows" on release)
        +-- lib.rs            # Tauri builder: plugins, state, 12 commands, setup
        +-- commands.rs       # #[tauri::command] IPC handlers — full clipboard + file + pairing impl
        +-- state.rs          # Managed state (DeviceIdentity, SessionManager, Storage, event forwarding)
```

### How it works (for web developers new to Tauri)

**Tauri** is like Electron but much lighter. Instead of bundling Chromium, it uses the operating system's native WebView (WebView2 on Windows, WebKit on macOS/Linux). This means:

- **Binary size**: ~7-10 MB vs Electron's ~100-150 MB
- **Memory usage**: Shares the OS web renderer, significantly lower RAM
- **Startup**: Near-instant since there's no Chromium to launch

**The IPC bridge** is the key concept. Your React code calls Rust functions through `invoke()`:

```typescript
// Frontend (TypeScript)
import { invoke } from "@tauri-apps/api/core";
const info = await invoke<DeviceInfo>("get_device_info");
```

```rust
// Backend (Rust)
#[tauri::command]
async fn get_device_info(state: State<'_, CosyncState>) -> Result<DeviceInfo, String> {
    // ... access cosync-core directly
}
```

Events flow the other direction — Rust emits them, React listens:

```rust
// Rust
app.emit("cosync://event", FrontendEvent::DeviceFound { ... });
```

```typescript
// React
listen<FrontendEvent>("cosync://event", (event) => { ... });
```

### Setup

#### Prerequisites

| Tool | Min Version | Purpose |
|---|---|---|
| Rust | 1.75+ | Core language + Tauri backend |
| Node.js | 18+ | Frontend build (Vite + React) |
| protoc | 3.21+ | Protobuf compiler (cosync-core build) |
| Tauri CLI | 2.x | `cargo install tauri-cli` |
| System libs | — | See platform-specific below |

#### Platform-specific system dependencies

**Ubuntu / Debian:**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libssl-dev
```

**Fedora:**
```bash
sudo dnf install webkit2gtk4.1-devel gtk3-devel \
  libappindicator-gtk3-devel librsvg2-devel openssl-devel
```

**macOS:** No extra system deps needed (Xcode CLI tools are sufficient).

**Windows:** No extra system deps needed (MSVC build tools are sufficient).

#### Step 1: Install Rust + protoc

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# protoc
PROTOC_VER="29.5"
curl -sL "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VER}/protoc-${PROTOC_VER}-linux-x86_64.zip" \
  -o /tmp/protoc.zip
unzip -qo /tmp/protoc.zip -d /tmp/protoc
cp /tmp/protoc/bin/protoc "$HOME/.cargo/bin/"
rm -rf /tmp/protoc /tmp/protoc.zip
```

#### Step 2: Install Tauri CLI

```bash
cargo install tauri-cli --version "^2"
```

#### Step 3: Install frontend dependencies

```bash
cd apps/desktop
npm install
```

#### Step 4: Development mode

```bash
# From apps/desktop/
npm run tauri dev
```

This starts Vite on port 1420 (hot reload for React) and opens the native Tauri window pointing at it. Rust changes trigger a recompile; frontend changes apply instantly via HMR.

#### Step 5: Production build

```bash
# From apps/desktop/
npm run tauri build
```

Output binaries are placed in `src-tauri/target/release/bundle/`.

#### Step 6: Run core tests

```bash
# From workspace root
cargo test -p cosync-core
```

## Offline Setup

For fully offline environments, pre-download all dependencies on an online machine:

```bash
# 1. Rust toolchain
rustup toolchain install stable --profile minimal

# 2. Cargo dependencies
cd cosync
cargo fetch --locked

# 3. Node.js (prebuilt binary)
curl -sO https://nodejs.org/dist/v20.11.0/node-v20.11.0-linux-x64.tar.xz

# 4. protoc (see Step 1 above)

# 5. Tauri system libs (apt download on Debian-based)
apt-get download libwebkit2gtk-4.1-dev libgtk-3-dev ...

# 6. Copy everything (cargo registry, node tarball, protoc binary) to offline machine
```

## Project Structure

```
cosync/
+-- Cargo.toml                 # Workspace root
+-- Cargo.lock
+-- README.md
+-- proto/
|   +-- cosync.proto           # Protobuf wire format
+-- crates/core/                # M0-M7: Core Rust library
|   +-- Cargo.toml
|   +-- build.rs               # prost-build
|   +-- src/
|       +-- lib.rs             # Crate root + UniFFI scaffold
|       +-- error.rs           # CosyncError (16 variants)
|       +-- hlc.rs             # Hybrid Logical Clock
|       +-- identity.rs        # Ed25519 + X.509 cert
|       +-- pairing.rs         # Pairing payload
|       +-- storage.rs         # SQLite CRUD
|       +-- state.rs           # ConnectionState enum
|       +-- envelope.rs        # Protobuf encode/decode
|       +-- wire.rs            # Generated protobuf code
|       +-- discovery.rs       # mDNS advertise/browse
|       +-- transport.rs       # QUIC + PinnedCertVerifier
|       +-- session.rs         # Session lifecycle
|       +-- clipboard.rs       # Clipboard sync + monitor
|       +-- file_transfer.rs   # 64KB chunked transfer
|       +-- notification.rs    # Notification mirror + filter
+-- apps/
    +-- desktop/               # M8: Tauri v2 desktop app
    |   +-- src-tauri/          # Rust backend (Tauri crate)
    |   +-- src/                # React/TypeScript frontend
    +-- mobile/                # M9: React Native (planned)
```

## Wire Protocol

```
+------------------------------------------+
|  4 bytes: payload length (big-endian u32) |
+------------------------------------------+
|  Protobuf Envelope                       |
|  +--------------------------------------+|
|  | sender_device_id: bytes (fingerprint) ||
|  | hlc_timestamp: bytes (JSON HLC)      ||
|  | payload: oneof                        ||
|  |   Heartbeat                           ||
|  |   PairingRequest / PairingAck         ||
|  |   ClipboardUpdate                     ||
|  |   FileMeta / FileChunk / FileAck      ||
|  |   NotificationEvent / Reply           ||
|  +--------------------------------------+|
+------------------------------------------+
```

## Security Model

1. **No CA** — Each device generates Ed25519 keypair + self-signed X.509 cert
2. **Pairing** — Devices pair over LAN by exchanging cert fingerprints
3. **Pinned Certs** — `PinnedCertVerifier` checks SHA-256 fingerprint, skips CA validation
4. **Loop Prevention** — HLC device_id check drops own echoed messages

## Milestones

| M | Description | Status |
|---|---|---|
| M0 | Scaffolding, workspace, proto | Done |
| M1 | Identity + storage + errors | Done |
| M2 | HLC + envelope encode/decode | Done |
| M3 | Discovery + transport (QUIC) | Done |
| M4 | Session manager + pairing | Done |
| M5 | Clipboard sync + monitor | Done |
| M6 | File transfer (chunked) | Done |
| M7 | Notification mirroring | Done |
| M8 | Desktop app (Tauri v2 + React) | Done |
| M9 | Mobile app (RN + UniFFI) | Planned |
| M10 | E2E testing + polish | Deferred |

## Desktop App — Registered IPC Commands

The Tauri backend exposes 12 commands to the React frontend via `invoke()`:

| Command | Description |
|---|---|
| `get_device_info` | Returns device name + SHA-256 fingerprint |
| `get_device_fingerprint` | Returns just the fingerprint |
| `get_connection_state` | Returns the current `ConnectionState` as a string |
| `start_discovery` | Starts mDNS browse, QUIC server, and event forwarding |
| `stop_discovery` | Shuts down the session + discovery service |
| `pair_with_device(ip, port, fp)` | Initiates QUIC pairing with a discovered peer |
| `unpair_device(device_id)` | Removes a device from the paired devices DB |
| `get_paired_devices` | Lists all previously paired devices from SQLite |
| `get_clipboard_history` | Returns the last 100 clipboard entries from SQLite |
| `send_clipboard(content)` | Broadcasts a text clipboard entry to all connected peers |
| `clear_clipboard_history` | Deletes all clipboard history from SQLite |
| `send_file(file_path)` | Chunks and streams a file to all connected peers |
| `open_file_in_folder(path)` | Opens the system file manager at the file's location |

### Event Flow

Rust emits typed events on the `cosync://event` channel. The React `useCosync()` hook listens and updates state:

```
SessionEvent (Rust)  →  FrontendEvent (serde)  →  Tauri emit  →  listen()  →  setState()
     │                        │                       │              │            │
     ClipboardReceived    ClipboardReceived         emit         .on()     history++
     FileIncoming         FileIncoming              emit         .on()     transfers++
     FileProgress         FileProgress              emit         .on()     progress%
     FileComplete         FileComplete              emit         .on()     status
     PeerPaired           PairingRequest            emit         .on()     (auto-accept in future)
     Error                Error                     emit         .on()     error banner
```

### UI Tabs

The app has three tabs in the main content area:

1. **Devices** — Start/stop mDNS scanning, view discovered peers, initiate pairing
2. **Clipboard** — Send text to paired devices, view sync history, copy entries locally, clear history
3. **Files** — Send files via system file picker or drag-and-drop, view transfer progress, open completed files in system folder