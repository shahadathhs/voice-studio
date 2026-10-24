# Building and versions

Voice Studio is built from source. There are no published installers: builds are not code-signed, so a
downloaded copy would be blocked or warned about by macOS and Windows. A copy you build yourself is not
quarantined and starts normally.

## Prerequisites

- [Rust](https://rustup.rs) (stable; `rust-toolchain.toml` adds rustfmt and clippy)
- a C++ compiler and [CMake](https://cmake.org) (whisper.cpp is compiled during the build)
- Node.js 22+ and [pnpm](https://pnpm.io)
- `make` (on Windows use Git Bash or WSL, or run the commands in the [Makefile](../Makefile) directly)
- Linux only: the [Tauri system packages](https://tauri.app/start/prerequisites/) and `libasound2-dev` (microphone)

`make setup` installs Rust and CMake if they are missing, the JavaScript dependencies and the git hooks.

## Build

```bash
git clone https://github.com/shahadathhs/voice-studio.git
cd voice-studio
make setup
make build            # the CLI and the desktop app, in release mode
```

| Output            | Where                                                                |
| ----------------- | -------------------------------------------------------------------- |
| `vtt` CLI         | `target/release/vtt` (or `cargo install --path crates/cli --locked`) |
| macOS app         | `target/release/bundle/macos/Voice Studio.app` and `bundle/dmg/`     |
| Windows installer | `target/release/bundle/msi/` and `bundle/nsis/`                      |
| Linux packages    | `target/release/bundle/deb/`, `bundle/rpm/` and `bundle/appimage/`   |

Install by copying the macOS app to `/Applications`, or running the Windows installer or Linux package you built.
`make build-cli` and `make build-desktop` build each part alone, and `make dev` runs the app with hot reload.

GPU on Windows and Linux: `cargo build --release -p vtt-cli --features vulkan` (needs the Vulkan SDK) or
`--features cuda` (needs the CUDA toolkit). The desktop crate forwards the same features. macOS uses Metal automatically.

## Versions

The version lives in the `VERSION` file and is copied by a script into everything else that carries one: the Rust
workspace (`Cargo.toml`, `Cargo.lock`), the Tauri config and the `package.json` files. Never edit those by hand.

```bash
make version v=0.2.0   # writes the files only; review with git diff, then commit
```

The script refuses to go backwards or to repeat the current version.

## Checks

CI (`.github/workflows/ci.yml`) runs the same checks as `make check` on Linux, macOS and Windows for every pull
request and every push to `main`. It builds and publishes nothing.

## Not set up

- **Prebuilt installers.** They need an Apple Developer account (signing and notarization) and a Windows
  code-signing certificate. Until then, build from source.
- **Updater.** The app does not check for or install updates.
