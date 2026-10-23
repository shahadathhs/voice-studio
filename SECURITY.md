# Security Policy

## Supported versions

Only the latest release receives security fixes. Nothing has been released yet.

## Reporting a vulnerability

Please **do not open a public issue** for security problems. Use GitHub's private vulnerability
reporting for this repository (Security tab → Report a vulnerability), or contact the maintainer
directly. Include:

- a description of the problem and its impact
- steps to reproduce
- a suggested fix, if you have one

What to expect: acknowledgement within 48 hours, an initial assessment within a week, and public
disclosure after a fix is released. Severity guides the timeline.

## Design principles

- **Offline and private.** Audio, text and transcripts stay on the user's computer. There is no
  telemetry and no account.
- **Network use is limited and visible.** The only network access is downloading models and checking
  for updates. Model downloads are planned to be verified against SHA-256 checksums from the catalog.
- **Memory safety.** The code is Rust, and `unsafe` is forbidden in this repository's crates.
- **No panics on bad input.** Malformed files return errors with documented exit codes; `unwrap` and
  `expect` are denied by lint outside tests.
- **Least surprise for files.** Inputs are read in place; outputs are written only where the user asks.

## Dependencies

Dependabot proposes updates weekly. Adding `cargo audit` and `cargo deny` (advisories and license
checks) to CI is planned.

## Questions

For general security questions, open an issue with the `security` label (without vulnerability details).
