#!/usr/bin/env bash
set -euo pipefail

mobile_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "$mobile_dir/../.." && pwd)"
output_dir="$mobile_dir/android/app/src/main/jniLibs"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build the Rust mobile bridge." >&2
  exit 1
fi

if ! cargo ndk --version >/dev/null 2>&1; then
  echo "cargo-ndk is required. Install with: cargo install cargo-ndk" >&2
  exit 1
fi

rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi \
  i686-linux-android \
  x86_64-linux-android

rm -rf "$output_dir"
mkdir -p "$output_dir"

cd "$repo_root"
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86 \
  -t x86_64 \
  -o "$output_dir" \
  build -p fnzero-safe-mobile-bridge --release

find "$output_dir" -name "libfnzero_safe_mobile_bridge.so" -print
