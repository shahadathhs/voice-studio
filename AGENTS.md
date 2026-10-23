# Voice Studio — guide for AI coding agents

Offline speech-to-text and text-to-speech as a cross-platform desktop app and a CLI. Rust core, Tauri 2
shell, Svelte 5 + TypeScript UI. Status: early development; see `docs/product/roadmap.md`.

Read first: `docs/README.md` (index), `docs/guidelines.md` (the rules below in full).

## Layout

- `crates/core` (`vtt-core`): types, errors, exporters, engine traits, model catalog, paths, job events.
- `crates/cli` (`vtt-cli`, binary `vtt`): clap front end and black-box tests.
- `desktop/src-tauri` (`vtt-desktop`): Tauri shell; thin commands that call `vtt-core` in-process.
- `desktop/src`: Svelte UI (`components/`, `pages/`, `lib/`, `styles/`).
- `catalog/models.json`: model catalog (Whisper, WeSpeaker, Kokoro) with pinned URLs, sizes, SHA-256 and licenses.
- `docs/`: `releasing.md`, `product/` (features, cli, roadmap), `architecture.md`, `design/` (mockups), `guidelines.md`.
- The earlier Python prototype lives only at git tag `python-final`.

## Commands

```bash
make setup          # pnpm install (also installs git hooks), builds the UI once
make dev            # desktop app with hot reload
make cli ARGS="..." # run the CLI
make check          # format-check, lint, types, UI build, clippy, tests: run before finishing
make test-e2e       # real model + real speech (needs VTT_TEST_MODEL, VTT_TEST_SPEECH); not part of check
make version v=X.Y.Z # write a new version into every file that carries it (no git)
make format         # rustfmt + Prettier (also run by the pre-commit hook on staged files)
make                # list all targets
```

Package manager is **pnpm** (workspace root holds git hooks; the UI is `desktop/`). `cargo` builds of
`vtt-desktop` need `desktop/dist`, which `make check` builds first. Git hooks (husky) run lint-staged on
commit and enforce Conventional Commit messages.

## Rules (full text in docs/guidelines.md)

- Files <= 300 lines, functions <= 50, <= 5 params, complexity limits, <= 3 bool fields per struct.
  Split by responsibility; never raise a limit or add an `allow` to pass.
- `vtt-core` never prints, parses argv or exits; the CLI and the desktop shell stay thin.
- Reuse and extract real duplication; no `utils`/`helpers` modules; share CLI arg groups and UI components.
- No `unwrap`/`expect`/panics/`todo!` outside tests. Unfinished features return `Error::NotImplemented`.
- UI text goes through `t()` with keys in `desktop/src/locales/` (English plus every other language; see docs/localization.md).
- Colours come from CSS variables in `desktop/src/styles/tokens.css`; UI logic that is not presentation
  belongs in Rust.
- Every behavior change has a test; CLI behavior is tested black-box in `crates/cli/tests`.
- Conventional Commits. Work on `dev`. Never run `make release` (it commits and pushes); the user does.

## Contracts to keep in sync

- CLI flags and exit codes: `docs/product/cli.md` and `vtt_core::Error::exit_code`.
- `vtt_core::Transcript` JSON is the interchange format (`--json`, `vtt export`, the app's store);
  changing it is a breaking change.
- Types crossing the Tauri boundary: `desktop/src-tauri/src/commands.rs` and `desktop/src/lib/types.ts`.
- Colour tokens: `desktop/src/styles/tokens.css` and `docs/design/mockups/src/common.css`.

## Undecided

Final product name (placeholders in `crates/core/src/paths.rs` and `desktop/src-tauri/tauri.conf.json`),
STT backend (whisper.cpp vs sherpa-onnx), model catalog entries and their license review.
