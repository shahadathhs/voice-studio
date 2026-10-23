#!/usr/bin/env bash
# Re-render every mockup in docs/design/mockups/src to PNG (needs Google Chrome).
set -euo pipefail
cd "$(dirname "$0")/mockups/src"
CHROME="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
for f in [0-9]*.html; do
  "$CHROME" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=2 \
    --window-size=1280,820 --virtual-time-budget=2000 \
    --screenshot="../${f%.html}.png" "file://$PWD/$f" >/dev/null 2>&1
done
# Dark-theme variants of key screens (common.js reads ?theme=dark)
for f in 01-transcribe-empty 04-record 05-speak 06-models 08-settings; do
  "$CHROME" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=2 \
    --window-size=1280,820 --virtual-time-budget=2000 \
    --screenshot="../${f}-dark.png" "file://$PWD/$f.html?theme=dark" >/dev/null 2>&1
done
