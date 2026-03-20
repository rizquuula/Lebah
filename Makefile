.PHONY: setup dev build clean setup-rust setup-node setup-tauri

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

clean:
	rm -rf dist node_modules
	cd src-tauri && cargo clean
