# Getting Started

## Prerequisites

### All Platforms

| Tool | Version | Install |
|---|---|---|
| Rust + Cargo | ≥ 1.88 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | ≥ 20 | [nodejs.org](https://nodejs.org) or via `nvm` |
| npm | ≥ 10 | bundled with Node.js |
| pnpm | ≥ 9 | `npm install -g pnpm` |
| protoc | ≥ 3.21 | see below — needed only to build `cosync-core` |

### protoc (Protocol Buffers Compiler)

`cosync-core/build.rs` calls `prost-build` which invokes `protoc` at compile time.

| OS | Command |
|---|---|
| macOS | `brew install protobuf` |
| Ubuntu / Debian | `apt install -y protobuf-compiler` |
| NixOS / Replit | `nix-env -i protobuf` (or add `pkgs.protobuf` to `replit.nix`) |
| Windows | Download from [github.com/protocolbuffers/protobuf/releases](https://github.com/protocolbuffers/protobuf/releases) |

### Tauri System Dependencies (Linux only)

On Linux, Tauri needs WebKit2GTK and related libraries. Install them once:

**Ubuntu / Debian (apt):**
```bash
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  pkg-config \
  libgtk-3-dev \
  libglib2.0-dev \
  libsoup-3.0-dev \
  libgdk-pixbuf2.0-dev
```

**NixOS / Replit (Nix):**
The Nix packages are installed automatically by the Replit environment via the package manager:
`webkitgtk_4_1`, `pkg-config`, `openssl`, `gtk3`, `librsvg`, `glib`, `pango`, `at-spi2-atk`, `libsoup_3`, `gdk-pixbuf`, `xorg.libX11`, `xorg.libXext`, `libayatana-appindicator`.

---

## Building the Desktop App

```bash
cd apps/desktop
npm install          # installs Vite + React + @tauri-apps/api + @tauri-apps/cli
npm run tauri build  # builds Vite frontend, then compiles Rust, then bundles
```

Target a specific bundle format:
```bash
npm run tauri build -- --bundles deb     # Linux .deb (recommended for Linux)
npm run tauri build -- --bundles appimage  # Linux AppImage
npm run tauri build -- --bundles dmg     # macOS disk image
npm run tauri build -- --bundles msi     # Windows installer
```

Output locations:
```
target/release/cosync-desktop              ← raw binary
target/release/bundle/deb/Cosync_*.deb    ← Linux .deb
target/release/bundle/dmg/Cosync_*.dmg    ← macOS .dmg
target/release/bundle/msi/Cosync_*.msi    ← Windows .msi
```

### Development Mode (with hot-reload)

```bash
cd apps/desktop
npm install
npm run tauri dev
```

This starts Vite's dev server on `localhost:1420` and launches the Tauri window pointing at it. Rust code changes require recompilation; React changes hot-reload instantly.

> **Linux heads-up:** Tauri `dev` needs a display. In a headless environment, prepend `DISPLAY=:0` or run inside a virtual framebuffer (`Xvfb`).

---

## Running the Landing Page

```bash
cd artifacts/web
pnpm install         # or: pnpm install from the workspace root
pnpm dev             # starts Vite dev server
```

Or from the workspace root:
```bash
pnpm --filter @workspace/web run dev
```

---

## Building cosync-core Only

```bash
# From workspace root — builds just the Rust library
cargo build -p cosync-core

# Release mode (optimised)
cargo build -p cosync-core --release

# Run library tests
cargo test -p cosync-core
```

---

## App Icons

The `assets/icons/logo.svg` file is the source for all Tauri icon variants.

To regenerate the icon set from a new SVG:
```bash
cd apps/desktop
# Requires ImageMagick for raster conversion
npm run tauri icon ../../assets/icons/logo.svg
```

This writes `src-tauri/icons/{32x32.png,128x128.png,128x128@2x.png,icon.icns,icon.ico}`.

---

## Workspace Root Scripts

```bash
pnpm build        # typecheck + build all web artifacts
pnpm typecheck    # TypeScript check across all packages
cargo build       # build all Rust workspace members
cargo test        # test all Rust workspace members
```

---

## Common Issues

### `protoc: command not found`
`prost-build` cannot find the Protocol Buffers compiler. Install `protoc` (see Prerequisites above).

### `notify-rust` MSRV error
`notify-rust ≥ 4.18` requires rustc ≥ 1.89. If you're on an older stable toolchain, pin it:
```bash
cargo update notify-rust --precise 4.11.3
```
Or install a newer Rust via `rustup update stable`.

### Blank preview on Replit
The Vite dev server must allow all hosts. Check that `vite.config.ts` has:
```ts
server: { host: "0.0.0.0" }
```
and that the workflow is running.
