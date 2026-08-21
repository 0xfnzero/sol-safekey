#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v flutter_rust_bridge_codegen >/dev/null 2>&1; then
  echo "flutter_rust_bridge_codegen is required." >&2
  echo "Install with: cargo install flutter_rust_bridge_codegen" >&2
  exit 1
fi

flutter_rust_bridge_codegen generate

echo "Flutter/Rust bridge generated."
