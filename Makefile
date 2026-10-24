# Voice Studio — one entry point for development tasks. Run `make` to list them.
# Windows: use Git Bash (or WSL); every target is a plain cargo/pnpm command you can run directly.

PNPM  := pnpm --dir desktop
CARGO := cargo

.DEFAULT_GOAL := help
.PHONY: notices help setup dev ui cli build build-cli build-desktop lint lint-rust lint-ui \
        format format-check typecheck test test-e2e check clean mockups screenshots version release

help: ## Show this help
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z_-]+:.*## / {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---- Setup -------------------------------------------------------------------

setup: ## Install Rust and CMake if missing, JS dependencies and git hooks, then build the UI once
	scripts/setup-toolchain.sh
	pnpm install
	$(PNPM) build

# ---- Run ---------------------------------------------------------------------

dev: ## Run the desktop app with hot reload
	$(PNPM) tauri dev

ui: ## Run only the UI in a browser (Tauri commands return stand-in data)
	$(PNPM) dev

cli: ## Run the CLI, e.g. make cli ARGS="export transcript.json -f srt"
	$(CARGO) run -p vtt-cli -- $(ARGS)

# ---- Build -------------------------------------------------------------------

build: build-cli build-desktop ## Build the CLI and the desktop installers (release)

build-cli: ## Build the release CLI binary (target/release/vtt)
	$(CARGO) build --release -p vtt-cli

build-desktop: ## Build the desktop app installers for this platform
	$(PNPM) tauri build

# ---- Quality -----------------------------------------------------------------

format: ## Format all code (rustfmt + Prettier)
	$(CARGO) fmt --all
	pnpm format

format-check: ## Check formatting without changing files
	$(CARGO) fmt --all --check
	pnpm exec prettier --check .

test-e2e: ## Transcribe real speech with a real model (set VTT_TEST_MODEL and VTT_TEST_SPEECH)
	$(CARGO) test -p vtt-core --release --test whisper_e2e -- --ignored

lint-rust: ## Clippy with warnings denied
	$(CARGO) lint

lint-ui: ## oxlint (type-aware, includes the file/function size limits)
	pnpm exec oxlint --type-aware .

lint: lint-rust lint-ui ## Run all linters

typecheck: ## svelte-check (TypeScript 7) + tsc
	pnpm typecheck

test: ## Run all Rust tests (includes the 300-line file limit)
	$(CARGO) test --workspace

# The Rust desktop shell embeds the built UI, so the UI is built before cargo runs.
check: format-check lint-ui typecheck ## Everything CI runs: format, lint, types, tests
	$(PNPM) build
	$(CARGO) lint
	$(CARGO) test --workspace

# ---- Versions and releases ---------------------------------------------------

version: ## Write a new version into every file that carries it (no git): make version v=0.2.0
	@v=$(v) scripts/bump-version.sh version

release: ## Bump, commit and push a release: make release v=0.2.0 (run by you, never automatic)
	@v=$(v) scripts/bump-version.sh release

tag: ## Tag main with the VERSION file's version and push it; CI then publishes the release
	@scripts/tag-release.sh

# ---- Housekeeping ------------------------------------------------------------

mockups: ## Re-render the design mockups to PNG (needs Google Chrome)
	docs/design/render.sh

screenshots: ## Re-render the README screenshots from the real UI with sample data (needs Google Chrome)
	$(PNPM) build
	node scripts/screenshots.mjs

notices: ## Regenerate THIRD_PARTY_NOTICES.md from the crates and the model catalog
	node scripts/notices.mjs

clean: ## Remove build output (keeps dependencies)
	$(CARGO) clean
	rm -rf desktop/dist
