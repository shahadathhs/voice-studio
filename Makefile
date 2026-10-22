# Voice Studio — one entry point for development tasks. Run `make` to list them.
# Windows: use Git Bash (or WSL); every target is a plain cargo/pnpm command you can run directly.

PNPM  := pnpm --dir desktop
CARGO := cargo

.DEFAULT_GOAL := help
.PHONY: help setup dev ui cli build build-cli build-desktop lint lint-rust lint-ui \
        format format-check typecheck test check clean mockups

help: ## Show this help
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z_-]+:.*## / {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---- Setup -------------------------------------------------------------------

setup: ## Install JS dependencies and git hooks, then build the UI once
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

# ---- Housekeeping ------------------------------------------------------------

mockups: ## Re-render the design mockups to PNG (needs Google Chrome)
	docs/design/render.sh

clean: ## Remove build output (keeps dependencies)
	$(CARGO) clean
	rm -rf desktop/dist
