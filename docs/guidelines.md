# Code guidelines

These rules keep the codebase small, readable and easy to change. Most are enforced by tooling, so
a green `cargo fmt --all --check && cargo lint && cargo test --workspace` means you are within them.

## Limits

| Rule                              | Limit | Enforced by                                                     |
| --------------------------------- | ----- | --------------------------------------------------------------- |
| Lines per file (including tests)  | 300   | `crates/core/tests/code_guidelines.rs` (runs in `cargo test`)   |
| Lines per function                | 50    | clippy `too_many_lines` (`clippy.toml`)                         |
| Parameters per function           | 5     | clippy `too_many_arguments`                                     |
| Cognitive complexity per function | 15    | clippy `cognitive_complexity`                                   |
| Boolean fields per struct         | 3     | clippy `struct_excessive_bools` (group related options instead) |
| Line width                        | 100   | `rustfmt.toml`                                                  |

If you hit a limit, the fix is almost always to split by responsibility, not to compress the code.
Do not raise a limit or add an `#[allow]` to get a change through; if a rule is wrong for a case,
discuss it in the PR and change the rule everywhere.

## Structure

- **One responsibility per module.** Name modules by what they do (`export::cues`, `models`, `paths`),
  never `utils`, `helpers`, `common` or `misc`.
- **`vtt-core` has no UI.** It never prints, never reads `argv`, never exits the process. It returns
  values and errors. Front ends (CLI, desktop) decide how to show them.
- **Front ends stay thin.** The CLI parses arguments and calls core. If logic would be needed by the
  desktop app too, it belongs in core.
- **Pure functions where possible.** Exporters take a transcript and return a string; they do no I/O.
- **Engines sit behind traits** (`SttEngine`, `TtsEngine`, `Diarizer`) so backends can be swapped and
  tests can use fakes.

## Reuse

- Search before you write. If something close exists, extend it.
- When the same logic appears a second time for the same reason, extract it (see
  `export::non_empty` and `export::speaker_label`, shared by every exporter).
- Share option groups instead of copying flags (see `OutputArgs` and `SpeakerArgs` in the CLI).
- Do not abstract speculatively. Extract on real duplication, not on a guess about the future.

## Clean code

- Names say what a thing is or does. Prefer a longer clear name to a short cryptic one.
- Early returns over deep nesting. Prefer iterators over index loops.
- Comments explain _why_, not _what_. Delete commented-out code; git remembers it.
- No dead code, no unused parameters, no `todo!()`/`unimplemented!()` (linted). Unfinished features
  return `Error::NotImplemented` so users get a clear message and exit code.
- Every public item has a doc comment saying what it is for.
- Magic numbers become named constants.

## Errors and safety

- No `unwrap()`, `expect()` or panics in non-test code (linted). Return `Result`.
- Errors carry a kind that maps to a documented exit code (`vtt_core::Error::exit_code`).
- `unsafe` is forbidden in our crates. If a dependency needs it, that is the dependency's concern.
- Never log or transmit user audio or text. The only network use is downloading models and update
  checks, and both are visible to the user.

## Tests

- Unit tests live next to the code (`#[cfg(test)] mod tests`). CLI behavior is covered by black-box
  tests in `crates/cli/tests`.
- Test behavior and the documented contract (output, exit codes), not implementation details.
- Every bug fix ships with a regression test that fails without the fix.
- Tests must not need network, GPU, microphone or large models. Use small fixtures or fakes.

## Dependencies

- Prefer the standard library. Add a crate only when it clearly pays for itself; say why in the PR.
- Licenses must be permissive (MIT, Apache-2.0, BSD, ISC, MPL-2.0 at most). No GPL/AGPL/non-commercial.
- Pin nothing exotic; Dependabot keeps versions current.

## Git

- [Conventional Commits](https://www.conventionalcommits.org): `feat:`, `fix:`, `docs:`, `refactor:`,
  `test:`, `chore:`, `ci:`. Mark breaking changes with `!` or a `BREAKING CHANGE:` footer.
- Small, focused commits and PRs. One concern per PR.
- Branches: work on `dev`; `main` is the stable branch.

## Desktop UI (Svelte + TypeScript)

Run from the repo root with `make lint-ui typecheck format-check` (or `pnpm lint`, `pnpm typecheck`).

| Rule                    | Limit                                       | Enforced by                             |
| ----------------------- | ------------------------------------------- | --------------------------------------- |
| Lines per file          | 300                                         | oxlint `max-lines`                      |
| Lines per function      | 50                                          | oxlint `max-lines-per-function`         |
| Parameters per function | 5                                           | oxlint `max-params`                     |
| Cyclomatic complexity   | 10                                          | oxlint `complexity`                     |
| Safety                  | no `any`, no floating promises, `===`       | oxlint (type-aware via oxlint-tsgolint) |
| Formatting              | 100 columns, double quotes, trailing commas | Prettier (+ svelte plugin)              |
| Types                   | strict TypeScript 7                         | `svelte-check --tsgo`, `tsc`            |

**Why oxlint and not ESLint:** the project uses TypeScript 7 (the native compiler), which has no
JavaScript compiler API for `typescript-eslint` to use. oxlint with `oxlint-tsgolint` is the supported
type-aware linter for it. The tooling matches the sibling Algomind project. `svelte-check` needs TypeScript 6
installed next to TypeScript 7 (aliased as `@typescript/native`) and runs with `--tsgo`; `.svelte-check/` is
generated output and is ignored by git, Prettier and oxlint.

Config lives at the repo root: `.oxlintrc.json`, `prettier.config.mjs`, `.prettierignore`, `.lintstagedrc.json`,
`commitlint.config.mjs`. Do not add per-package copies.

- Components are small, typed, and presentational. State lives in `lib/*.svelte.ts` stores; talking to
  Rust goes through `lib/api.ts` only.
- Reuse the existing components (`Segmented`, `PageHeader`, `Notice`, `EmptyState`, `Icon`) before
  writing new markup. Colours come from CSS variables in `styles/tokens.css`, never hard-coded.
- No user-facing text in markup or stores: add a key to `locales/en/` and every other locale, and use `t()`
  (see [localization.md](localization.md)). Use `Intl` for dates, numbers and language names.
- Anything that is logic rather than presentation belongs in Rust (`vtt-core`) behind a Tauri command.
- Accessibility is part of done: real buttons, labels, focus styles, keyboard operation, AA contrast in
  both themes.
