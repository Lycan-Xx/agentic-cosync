# Cosync — LAN Device Sync

**Clipboard, Files, Notifications — synced across your devices over your local network.**

Cosync is a peer-to-peer device synchronization system that works entirely over LAN (no cloud, no accounts). It uses QUIC for fast, multiplexed transport, mDNS for zero-config peer discovery, and QR-code-based pairing with pinned self-signed certificates for security.

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
| Security | QR-exchanged pinned self-signed certs (Ed25519 + X.509) |
| Ordering | Hybrid Logical Clock (HLC) for causal consistency |
| Storage | SQLite via rusqlite (WAL mode) |
| Wire Format | Protobuf via prost |
| Mobile Bridge | UniFFI -> Kotlin -> React Native native module |

## Offline Setup

### Prerequisites

| Tool | Min Version | Purpose |
|---|---|---|
| Rust | 1.75+ | Core language |
| Node.js | 18+ | Desktop (Tauri) and Mobile (Expo) |
| protoc | 3.21+ | Protobuf compiler |
| Android SDK | API 33+ | Mobile builds (with NDK) |

### Step 1: Install Rust

**Online:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

**Offline (prebuilt binary):**
```bash
# Download on an online machine:
curl -sO https://static.rust-lang.org/dist/rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz
# Transfer via USB, then:
tar xzf rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz
cd rust-1.75.0-x86_64-unknown-linux-gnu
./install.sh --prefix=$HOME/.rust-local
export PATH="$HOME/.rust-local/bin:$PATH"
```

### Step 2: Install protoc (Offline)

```bash
# Download on online machine:
curl -sL https://github.com/protocolbuffers/protobuf/releases/download/v28.3/protoc-28.3-linux-x86_64.zip -o protoc.zip
unzip protoc.zip -d /tmp/protoc
# Transfer protoc/bin/protoc binary to offline machine:
cp /tmp/protoc/bin/protoc ~/.local/bin/protoc
chmod +x ~/.local/bin/protoc
export PATH="$HOME/.local/bin:$PATH"
```

### Step 3: Pre-fetch Cargo Dependencies (Online)

```bash
cd cosync
cargo fetch --locked
# Copy entire ~/.cargo/ directory to offline machine
```

### Step 4: Build

```bash
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
cd cosync

# Check compilation
cargo check -p cosync-core

# Run all 23 tests
cargo test -p cosync-core

# Release build
cargo build -p cosync-core --release
```

### Step 5: Install Node.js (Offline, for M8/M9)

```bash
# Download on online machine:
curl -sO https://nodejs.org/dist/v20.11.0/node-v20.11.0-linux-x64.tar.xz
# Transfer, extract on offline machine:
tar xf node-v20.11.0-linux-x64.tar.xz -C ~/.local/
export PATH="$HOME/.local/node-v20.11.0-linux-x64/bin:$PATH"
```

## Project Structure

```
cosync/
+-- Cargo.toml                 # Workspace root
+-- README.md
+-- proto/
|   +-- cosync.proto           # Protobuf wire format
+-- crates/core/
|   +-- Cargo.toml
|   +-- build.rs               # prost-build
|   +-- src/
|       +-- lib.rs             # Crate root + UniFFI scaffold
|       +-- error.rs           # CosyncError (16 variants)
|       +-- hlc.rs             # Hybrid Logical Clock
|       +-- identity.rs        # Ed25519 + X.509 cert
|       +-- pairing.rs         # QR pairing payload
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
    +-- desktop/               # Tauri v2 (planned)
    +-- mobile/                # React Native / Expo bare (planned)
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
2. **QR Pairing** — Scan QR with device name, cert fingerprint, IP, port, 32-byte token
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
| M8 | Desktop app (Tauri + React) | Planned |
| M9 | Mobile app (RN + UniFFI) | Planned |
| M10 | E2E testing + polish | Deferred |