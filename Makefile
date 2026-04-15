.PHONY: help setup dev build run run-released clean clean-soft setup-rust setup-node setup-tauri tree test e2e e2e-install e2e-ui
.DEFAULT_GOAL := help

help:
	@echo "Lebah - Claude Code Orchestrator"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "  tree    Show file tree for .ts/.svelte/.rs files with line counts"
	@echo "  setup   Install dependencies (Tauri CLI, npm packages, cargo deps)"
	@echo "  dev     Run in development mode"
	@echo "  build        Build for production (with bundles)"
	@echo "  run          Build (no bundles) and run"
	@echo "  run-released Run the existing release build without rebuilding"
	@echo "  test    Run all checks (Rust tests, Svelte type check, Cargo check)"
	@echo "  e2e          Run Playwright E2E suite (headless)"
	@echo "  e2e-ui       Run Playwright E2E suite in UI mode"
	@echo "  e2e-install  Install Playwright browsers (first-time setup)"
	@echo "  clean       Clean all build artifacts and dependencies"
	@echo "  clean-soft  Clear caches only (Vite, Cargo incremental)"
	@echo ""

tree:
	@find src src-tauri/src -type f \( -name "*.ts" -o -name "*.svelte" -o -name "*.rs" \) \
		| sort \
		| while read f; do \
			lines=$$(wc -l < "$$f"); \
			printf "%-55s %4d lines\n" "$$f" "$$lines"; \
		done

setup: setup-rust setup-node setup-tauri

setup-rust:
	@echo "Installing Rust dependencies..."
	cd src-tauri && cargo fetch

setup-node:
	@echo "Installing Node dependencies..."
	npm install

setup-tauri:
	@echo "Installing Tauri CLI..."
	cargo install tauri-cli --version "^2"

dev:
	cargo tauri dev --no-watch

build:
	cargo tauri build

run:
	cargo tauri build --no-bundle
	"$${CARGO_TARGET_DIR:-src-tauri/target}/release/lebah"

run-released:
	"$${CARGO_TARGET_DIR:-src-tauri/target}/release/lebah"

test:
	@echo "Running Rust checks..."
	cd src-tauri && cargo check
	@echo "Running Rust tests..."
	cd src-tauri && cargo test
	@echo "Running Svelte type check..."
	npx svelte-check --tsconfig tsconfig.json
	@echo "All checks passed!"

e2e-install:
	npm run e2e:install

e2e:
	npm run e2e

e2e-ui:
	npm run e2e:ui

clean-soft:
	@echo "Clearing caches..."
	rm -rf node_modules/.vite dist
	cd src-tauri && cargo clean --profile dev

clean: clean-soft
	@echo "Cleaning all..."
	rm -rf node_modules
	cd src-tauri && cargo clean
