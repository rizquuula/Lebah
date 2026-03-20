.PHONY: help setup dev build clean clean-soft setup-rust setup-node setup-tauri
.DEFAULT_GOAL := help

help:
	@echo "LebahTempa - Claude Code Orchestrator"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "  setup   Install dependencies (Tauri CLI, npm packages, cargo deps)"
	@echo "  dev     Run in development mode"
	@echo "  build   Build for production"
	@echo "  clean       Clean all build artifacts and dependencies"
	@echo "  clean-soft  Clear caches only (Vite, Cargo incremental)"
	@echo ""

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
	cargo tauri dev

build:
	cargo tauri build

clean-soft:
	@echo "Clearing caches..."
	rm -rf node_modules/.vite dist
	cd src-tauri && cargo clean --profile dev

clean: clean-soft
	@echo "Cleaning all..."
	rm -rf node_modules
	cd src-tauri && cargo clean
