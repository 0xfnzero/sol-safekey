#!/bin/bash

# Build the CLI with full features.
cargo build --release -p fnzero-safe-core --features full

# Copy the binary to project root
cp -rf build-cache/release/fnzero-safe ./
