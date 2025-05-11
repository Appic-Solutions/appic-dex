# Makefile

# Build target
build:
	@echo "Building Appic Dex..."
	cargo build --release --target wasm32-unknown-unknown --package appic_dex
	candid-extractor target/wasm32-unknown-unknown/release/appic_dex.wasm > appic_dex.did
	cp target/wasm32-unknown-unknown/release/appic_dex.wasm src/tests/integration/wasm


test:
	@echo "Starting the test..."
	@echo "Building Rust project..."
	cargo build --release --target wasm32-unknown-unknown --package appic_dex
	candid-extractor target/wasm32-unknown-unknown/release/appic_dex.wasm > appic_dex.did
	cp target/wasm32-unknown-unknown/release/appic_dex.wasm src/tests/integration/wasm
	cargo test
