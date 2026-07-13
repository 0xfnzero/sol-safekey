SHELL := /bin/bash

.DEFAULT_GOAL := help

.PHONY: help install ui-dev ui-build api-build desktop-dev desktop-build check

help:
	@echo "Sol SafeKey workspace commands"
	@echo "  make install        Install UI dependencies and prefetch Rust crates"
	@echo "  make ui-dev         Start Next.js and the local Rust API"
	@echo "  make ui-build       Build the static web UI"
	@echo "  make api-build      Build the UI API release binary"
	@echo "  make desktop-dev    Start the Tauri desktop app"
	@echo "  make desktop-build  Build the Tauri desktop app"
	@echo "  make check          Run Rust, lint, and TypeScript checks"

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
	cd ui && npm run lint
	cd ui && npm exec tsc -- --noEmit
