.PHONY: help setup dev build run clean clean-soft setup-rust setup-node setup-tauri tree test
.DEFAULT_GOAL := help

help:
	@echo "Lebah - Claude Code Orchestrator"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "  tree    Show file tree for .ts/.svelte/.rs files with line counts"
	@echo "  setup   Install dependencies (Tauri CLI, npm packages, cargo deps)"
	@echo "  dev     Run in development mode"
	@echo "  build   Build for production"
	@echo "  run     Run the release build"
	@echo "  test    Run all checks (Rust tests, Svelte type check, Cargo check)"
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
	RUST_LOG=debug cargo tauri dev --no-watch

build:
	cargo tauri build

run: build
	RUST_LOG=debug "$${CARGO_TARGET_DIR:-src-tauri/target}/release/lebah"

test:
	@echo "Running Rust checks..."
	cd src-tauri && cargo check
	@echo "Running Rust tests..."
	cd src-tauri && cargo test
	@echo "Running Svelte type check..."
	npx svelte-check --tsconfig tsconfig.json
	@echo "All checks passed!"

clean-soft:
	@echo "Clearing caches..."
	rm -rf node_modules/.vite dist
	cd src-tauri && cargo clean --profile dev

clean: clean-soft
	@echo "Cleaning all..."
	rm -rf node_modules
	cd src-tauri && cargo clean
