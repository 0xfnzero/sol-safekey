#!/bin/bash

# Build with full features (includes CLI)
cargo build --release --features full

# Copy the binary to project root
cp -rf build-cache/release/sol-safekey ./
