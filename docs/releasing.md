# Releasing

The version lives in the `VERSION` file and is copied by a script into everything else that carries
one: the Rust workspace (`Cargo.toml`, `Cargo.lock`), the Tauri config, and both `package.json` files.
Never edit those by hand.

## Cut a release

1. On `dev`, with a clean working tree: `make release v=0.2.0`. This writes the version everywhere,
   commits `chore(release): v0.2.0` and pushes. Use `make version v=0.2.0` to change the files only.
2. Merge `dev` into `main` through a pull request.
3. On `main`, up to date and clean: `make tag`. It tags the commit `v0.2.0` from the `VERSION` file and
   pushes the tag.
4. The `CI` workflow (`.github/workflows/ci.yml`, the only workflow) runs the checks on the tag, then
   builds installers on Linux, macOS and Windows and publishes `v0.2.0` with generated notes, SHA-256
   files and the `vtt` command-line binary for each platform. The tag must match `VERSION`.

Pushes to `main` without a tag only run the checks. Nothing is released until a tag is pushed.

The script refuses to go backwards, to repeat the current version, or to run with uncommitted changes.

## What is not set up yet

- **Signing.** macOS builds are not notarized and Windows builds are not code-signed, so the operating
  system will warn on first launch. Needs an Apple Developer account and a signing certificate.
- **Updater.** The app does not check for or install updates.
- **GPU builds.** Release builds use the CPU on Windows and Linux (Metal on macOS). Vulkan and CUDA are
  opt-in cargo features that need their SDKs; see the README.
