.PHONY: build release test clean install

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clean:
	cargo clean

install:
	cargo install --path .

help:
	@echo "Available targets:"
	@echo "  build    - Build debug version"
	@echo "  release  - Build release version"
	@echo "  test     - Run tests"
	@echo "  clean    - Clean build artifacts"
	@echo "  install  - Install binary locally"
