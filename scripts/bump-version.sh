#!/usr/bin/env bash
# Bump the product version.
#
#   make version v=0.2.0    write the version into every file that carries it (no git)
#   make release v=0.2.0    the same, then commit and push (you run this; it is not automatic)
#
# The VERSION file is the source of truth for release builds. This script also updates the Rust
# workspace, the Tauri config and the two package.json files, so they never drift. When the bump
# reaches main, `make tag` tags it and CI publishes v0.2.0 (see docs/releasing.md).
set -euo pipefail

cd "$(dirname "$0")/.."

mode="${1:-version}" # "version" (files only) or "release" (commit and push)
v="${v:-}"

[[ "$v" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
  echo "Usage: make ${mode} v=0.2.0   (a plain MAJOR.MINOR.PATCH version)" >&2
  exit 1
}

current="$(tr -d '[:space:]' < VERSION)"
if [[ "$v" == "$current" ]]; then
  echo "VERSION is already ${v}; nothing to do." >&2
  exit 1
fi
newest="$(printf '%s\n%s\n' "$current" "$v" | sort -V | tail -n 1)"
if [[ "$newest" != "$v" ]]; then
  echo "Refusing to go backwards: ${current} -> ${v}." >&2
  exit 1
fi

files=(VERSION Cargo.toml Cargo.lock package.json desktop/package.json desktop/src-tauri/tauri.conf.json)

if [[ "$mode" == "release" ]]; then
  # The release commit must contain only the version bump.
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Working tree has uncommitted changes. Commit or stash them first." >&2
    exit 1
  fi
fi

printf '%s\n' "$v" > VERSION

# Only the [workspace.package] version line (the first `version =`) changes.
node -e '
  const fs = require("node:fs");
  const [file, version] = process.argv.slice(1);
  const text = fs.readFileSync(file, "utf8");
  fs.writeFileSync(file, text.replace(/^version = ".*"$/m, `version = "${version}"`));
' Cargo.toml "$v"

for json in package.json desktop/package.json desktop/src-tauri/tauri.conf.json; do
  node -e '
    const fs = require("node:fs");
    const [file, version] = process.argv.slice(1);
    const json = JSON.parse(fs.readFileSync(file, "utf8"));
    json.version = version;
    fs.writeFileSync(file, JSON.stringify(json, null, 2) + "\n");
  ' "$json" "$v"
done

# Refresh the workspace crates' versions in Cargo.lock without touching dependencies.
cargo update --workspace --offline >/dev/null

echo "Version ${current} -> ${v} in: ${files[*]}"

if [[ "$mode" == "release" ]]; then
  git add "${files[@]}"
  git commit -m "chore(release): v${v}"
  git push origin "$(git rev-parse --abbrev-ref HEAD)"
  echo "v${v} pushed. Merge to main and the pipeline publishes the release."
else
  echo "Files updated. Review with git diff; run 'make release v=${v}' to commit and push."
fi
