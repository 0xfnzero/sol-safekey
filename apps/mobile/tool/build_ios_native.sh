#!/usr/bin/env bash
set -euo pipefail

mobile_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "$mobile_dir/../.." && pwd)"
bridge_dir="$repo_root/crates/mobile-bridge"
build_dir="$mobile_dir/ios/Frameworks/FnzeroSafeMobileBridge"
headers_dir="$build_dir/include"
xcframework="$mobile_dir/ios/Frameworks/FnzeroSafeMobileBridge.xcframework"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build the Rust mobile bridge." >&2
  exit 1
fi

if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "xcodebuild is required to package the iOS bridge." >&2
  exit 1
fi

rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

export IPHONEOS_DEPLOYMENT_TARGET=15.0
export IPHONESIMULATOR_DEPLOYMENT_TARGET=15.0

cd "$repo_root"
target_dir="$(cargo metadata --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
if [[ -z "$target_dir" ]]; then
  echo "Failed to resolve Cargo target directory." >&2
  exit 1
fi

cargo build -p fnzero-safe-mobile-bridge --release --target aarch64-apple-ios
cargo build -p fnzero-safe-mobile-bridge --release --target aarch64-apple-ios-sim
cargo build -p fnzero-safe-mobile-bridge --release --target x86_64-apple-ios

rm -rf "$build_dir" "$xcframework"
mkdir -p "$headers_dir"

cp "$mobile_dir/ios/Runner/bridge_generated.h" "$headers_dir/bridge_generated.h"
cat > "$headers_dir/module.modulemap" <<'EOF'
module FnzeroSafeMobileBridge {
  header "bridge_generated.h"
  export *
}
EOF

lipo -create \
  "$target_dir/aarch64-apple-ios-sim/release/libfnzero_safe_mobile_bridge.a" \
  "$target_dir/x86_64-apple-ios/release/libfnzero_safe_mobile_bridge.a" \
  -output "$build_dir/libfnzero_safe_mobile_bridge_sim.a"

cp "$target_dir/aarch64-apple-ios/release/libfnzero_safe_mobile_bridge.a" \
  "$build_dir/libfnzero_safe_mobile_bridge_ios.a"

xcodebuild -create-xcframework \
  -library "$build_dir/libfnzero_safe_mobile_bridge_ios.a" -headers "$headers_dir" \
  -library "$build_dir/libfnzero_safe_mobile_bridge_sim.a" -headers "$headers_dir" \
  -output "$xcframework"

echo "$xcframework"
