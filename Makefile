.PHONY: build test bindings clean help

build:
	cargo build --release --target wasm32-unknown-unknown

test:
	cargo test

bindings: build
	mkdir -p sdk/generated
	stellar contract bindings typescript \
		--wasm target/wasm32-unknown-unknown/release/mergemint_contracts.wasm \
		--output-dir sdk/generated

clean:
	cargo clean
	rm -rf sdk/generated

help:
	@echo "Available targets:"
	@echo "  make build     - Build WASM contract"
	@echo "  make test      - Run test suite"
	@echo "  make bindings  - Generate TypeScript bindings"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make help      - Show this help"
