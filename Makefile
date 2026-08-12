SHELL := /bin/bash

RELEASE_NODE_VERSION := 20.19.5
RELEASE_NPM_VERSION := 10.8.2
RELEASE_CARGO_AUDIT_VERSION := 0.22.1

.DEFAULT_GOAL := help

.PHONY: help install ui-dev ui-build api-build desktop-dev desktop-build check release-check

help:
	@echo "Sol SafeKey workspace commands"
	@echo "  make install        Install UI dependencies and prefetch Rust crates"
	@echo "  make ui-dev         Start Next.js and the local Rust API"
	@echo "  make ui-build       Build the static web UI"
	@echo "  make api-build      Build the UI API release binary"
	@echo "  make desktop-dev    Start the Tauri desktop app"
	@echo "  make desktop-build  Build the Tauri desktop app"
	@echo "  make check          Run Rust, lint, and TypeScript checks"
	@echo "  make release-check  Run the complete non-deploying release gate"

install:
	cd ui && npm ci
	cargo fetch

ui-dev:
	cd ui && npm run dev:stack

ui-build:
	cd ui && npm run build

api-build: ui-build
	cargo build --release -p sol-safekey-ui

desktop-dev:
	cd ui && npm run desktop:dev

desktop-build:
	cd ui && npm run desktop:build

check:
	cargo check --workspace
	cd ui && npm run test:dependency-compat
	cd ui && npm run test:devnet-airdrop
	cd ui && npm run test:program-deploy
	cd ui && npm run lint
	cd ui && npm exec tsc -- --noEmit

release-check:
	@command -v cargo-audit >/dev/null || { echo "cargo-audit $(RELEASE_CARGO_AUDIT_VERSION) is required: cargo install cargo-audit --locked --version $(RELEASE_CARGO_AUDIT_VERSION)" >&2; exit 1; }
	@test "$$(cargo audit --version | awk '{print $$NF}')" = "$(RELEASE_CARGO_AUDIT_VERSION)" || { echo "cargo-audit must be exactly $(RELEASE_CARGO_AUDIT_VERSION)" >&2; exit 1; }
	@test "$$(node -p 'process.versions.node')" = "$(RELEASE_NODE_VERSION)" || { echo "Node.js must be exactly $(RELEASE_NODE_VERSION)" >&2; exit 1; }
	@test "$$(npm --version)" = "$(RELEASE_NPM_VERSION)" || { echo "npm must be exactly $(RELEASE_NPM_VERSION)" >&2; exit 1; }
	cd ui && node scripts/assert-no-next-dev.cjs
	cargo fmt --all -- --check
	cargo test --locked --workspace
	cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
	cd ui && npm ci
	cd ui && npm run test:dependency-compat
	cd ui && npm run test:devnet-airdrop
	cd ui && npm run test:program-deploy
	cd ui && npm run lint
	cd ui && npm exec tsc -- --noEmit
	cd ui && npm run build
	cargo audit
	cd ui && npm audit --audit-level=high
