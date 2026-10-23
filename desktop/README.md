# Desktop app

Tauri 2 shell (`src-tauri/`, Rust) with a Svelte 5 + TypeScript UI (`src/`). The repo-level `make` targets wrap these. Architecture:
[docs/architecture.md](../docs/architecture.md). Rules: [docs/guidelines.md](../docs/guidelines.md).

```bash
pnpm install             # from the repo root (installs the whole workspace and git hooks)
pnpm tauri dev           # run the app with hot reload
pnpm dev                 # UI only, in a browser (Tauri commands return stand-in data)
pnpm typecheck          # svelte-check (TypeScript 7)  — lint/format run from the repo root: make lint-ui format-check
pnpm build               # frontend into dist/ (required before cargo builds the shell)
pnpm tauri build         # installers for the current platform
```

Linux needs the WebKitGTK development packages; see the
[Tauri prerequisites](https://tauri.app/start/prerequisites/).
