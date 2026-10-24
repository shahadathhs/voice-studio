# Releasing

The version lives in the `VERSION` file and is copied by a script into everything else that carries
one: the Rust workspace (`Cargo.toml`, `Cargo.lock`), the Tauri config, and both `package.json` files.
Never edit those by hand.

## Cut a release

1. On `dev`, with a clean working tree: `make release v=0.2.0`. This writes the version everywhere,
   commits `chore(release): v0.2.0` and pushes. Use `make version v=0.2.0` to change the files only.
2. Merge `dev` into `main` through a pull request.
3. The `CI` workflow (`.github/workflows/ci.yml`, the only workflow) runs the checks, then, because
   `VERSION` names a version with no release yet, builds installers
   on Linux, macOS and Windows and publishes `v0.2.0` with generated notes, SHA-256 files and the
   `vtt` command-line binary for each platform.

Pushing a `v*` tag does the same. Installers are built only after the checks pass. A main push whose
version already has a release only runs the checks.

The script refuses to go backwards, to repeat the current version, or to run with uncommitted changes.

## What is not set up yet

- **Signing.** macOS builds are not notarized and Windows builds are not code-signed, so the operating
  system will warn on first launch. Needs an Apple Developer account and a signing certificate.
- **Updater.** The app does not check for or install updates.
- **GPU builds.** Release builds use the CPU on Windows and Linux (Metal on macOS). Vulkan and CUDA are
  opt-in cargo features that need their SDKs; see the README.
