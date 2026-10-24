#!/usr/bin/env bash
# Tag main with the version in the VERSION file and push the tag. The CI workflow then builds and
# publishes the release (see docs/releasing.md). You run this; it is not automatic.
set -euo pipefail

cd "$(dirname "$0")/.."

version="$(tr -d '[:space:]' < VERSION)"
tag="v${version}"

[[ "$(git branch --show-current)" == "main" ]] || {
  echo "Run this on main, after the release pull request is merged." >&2
  exit 1
}
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Working tree has uncommitted changes." >&2
  exit 1
fi
git fetch --quiet origin main
[[ "$(git rev-parse HEAD)" == "$(git rev-parse origin/main)" ]] || {
  echo "main is not up to date with origin/main. Pull first." >&2
  exit 1
}
if git rev-parse --verify --quiet "refs/tags/${tag}" >/dev/null; then
  echo "Tag ${tag} already exists." >&2
  exit 1
fi

git tag -a "$tag" -m "Voice Studio ${tag}"
git push origin "$tag"
echo "Pushed ${tag}; the CI workflow will publish the release."
