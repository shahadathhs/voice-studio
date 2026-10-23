#!/bin/sh
# Installs what `make setup` needs besides JS dependencies: a C/C++ compiler, CMake (whisper.cpp is
# compiled during the build), the Rust toolchain and, on Linux, the Tauri system packages.
# Rust is installed per-user; system packages use Homebrew on macOS and apt on Debian/Ubuntu.
# Anything else is reported with the command to run.

set -eu

have() { command -v "$1" >/dev/null 2>&1; }

sudo_cmd() { if [ "$(id -u)" -eq 0 ]; then "$@"; else sudo "$@"; fi; }

system="$(uname -s)"

# --- C/C++ compiler ---------------------------------------------------------------------------
if have cc && have c++; then
  echo "compiler: found $(cc --version | head -n 1)"
elif [ "$system" = "Darwin" ]; then
  echo "compiler: not found. Run 'xcode-select --install', then run make setup again." >&2
  exit 1
else
  echo "compiler: not found, it is installed with the Linux packages below"
fi

# --- Linux: Tauri system packages and build tools -------------------------------------------------
if [ "$system" = "Linux" ] && have apt-get; then
  echo "linux: installing build tools and Tauri packages with apt"
  sudo_cmd apt-get update
  sudo_cmd apt-get install -y build-essential cmake curl pkg-config libssl-dev libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev libxdo-dev libasound2-dev
elif [ "$system" = "Linux" ]; then
  echo "linux: install build tools, CMake and the Tauri packages for your distribution:" >&2
  echo "  https://tauri.app/start/prerequisites/" >&2
fi

# --- CMake ------------------------------------------------------------------------------------
if have cmake; then
  echo "cmake: found $(cmake --version | head -n 1)"
elif [ "$system" = "Darwin" ] && have brew; then
  echo "cmake: not found, installing with Homebrew"
  brew install cmake
else
  echo "cmake: not found. Install it (macOS: brew install cmake, Windows: winget install Kitware.CMake)." >&2
  exit 1
fi

# --- Rust -------------------------------------------------------------------------------------
if have cargo; then
  echo "rust: found $(cargo --version)"
else
  echo "rust: not found, installing with rustup (official installer, per-user)"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  # A fresh install is only on PATH in new shells; load it so the steps below can find rustup.
  if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi
fi

if have rustup; then
  # rust-toolchain.toml pins the channel and components; this makes rustup fetch them now.
  rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install stable --profile minimal
  rustup component add rustfmt clippy >/dev/null
  echo "rust: ready (managed by rustup). If cargo was just installed, open a new terminal."
elif have rustfmt && have cargo-clippy; then
  # Rust installed another way (for example `brew install rust`) has no rustup but includes both.
  echo "rust: ready (not managed by rustup: found rustfmt and clippy; the version pin in"
  echo "      rust-toolchain.toml is not applied, so use a recent stable Rust)"
else
  echo "rust: cargo is installed without rustup and rustfmt or clippy is missing." >&2
  echo "      Install them (macOS: brew install rust) or switch to rustup: https://rustup.rs" >&2
  exit 1
fi
