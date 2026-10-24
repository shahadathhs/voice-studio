# Contributing

Thanks for helping. This project is small and aims to stay that way: short files, no duplicated logic,
and behavior covered by tests. By taking part you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Setup

Install [Rust](https://rustup.rs) (stable; `rust-toolchain.toml` adds rustfmt and clippy), a C++ compiler and
[CMake](https://cmake.org) (the speech engine, whisper.cpp, is compiled during the build), Node.js 22+,
[pnpm](https://pnpm.io) and `make`. On Linux, install the
[Tauri system packages](https://tauri.app/start/prerequisites/) and `libasound2-dev` (microphone recording). On Windows, use Git Bash or WSL for
`make`, or run the commands in the [Makefile](Makefile) directly.

```bash
git clone https://github.com/shahadathhs/voice-to-text.git
cd voice-to-text
git switch dev
make setup      # installs Rust/CMake if missing, pnpm install, git hooks, first UI build
make dev        # run the desktop app
```

`make` lists all targets. Git hooks (installed by `make setup`) format staged files on commit and reject
commit messages that are not [Conventional Commits](https://www.conventionalcommits.org).

## Before you open a pull request

```bash
make format     # rustfmt + Prettier
make check      # format-check, oxlint, svelte-check, UI build, clippy (warnings denied), all tests
```

CI runs the same checks on Linux, macOS and Windows.

## Guidelines

Read [docs/guidelines.md](docs/guidelines.md). In short:

- Files stay under 300 lines and functions under 50. Split by responsibility instead of compressing.
- `vtt-core` has no UI and never prints; the CLI and the desktop shell stay thin.
- Reuse before you write; extract real duplication; no `utils` dumping grounds.
- No `unwrap`, `expect`, panics, dead code or commented-out code.
- Every bug fix comes with a regression test.

## Pull requests

1. Branch from `dev`; keep the change focused on one concern.
2. Use [Conventional Commits](https://www.conventionalcommits.org) (`feat:`, `fix:`, `docs:` ...).
3. Fill in the PR template. Update `docs/product/cli.md` or `docs/product/features.md` if behavior or
   scope changes.
4. Be ready for review feedback; small PRs are reviewed faster.

## Reporting bugs and ideas

Use the issue templates. For bugs include your OS, app version, whether you used the desktop app or the
CLI, the exact command, and the output. For security problems do not open a public issue; see
[SECURITY.md](SECURITY.md).

## Adding a model or voice

Models go in `catalog/models.json` with a real download URL, SHA-256 and license. Only permissively
licensed models are accepted (see [catalog/README.md](catalog/README.md)).
