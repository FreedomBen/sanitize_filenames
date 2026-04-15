BINARY_NAME := sanitize_filenames
TARGET := x86_64-unknown-linux-musl
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)
DEB_PACKAGE := sanitize-filenames
DEB_ARCH := amd64

.PHONY: all build clean deps initialize run test rpm deb arch-pkg alpine-apk \
	release-binary release-rpm release-deb release-arch release-alpine install help

all: build ## Build the release binary (default)

help: ## Show this help message
	@echo "Usage: make [target]"
	@echo
	@echo "Targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-16s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

initialize: ## Install Rust target, fetch deps, and build
	rustup target add $(TARGET)
	$(MAKE) deps
	$(MAKE) build

deps: ## Fetch cargo dependencies for the target
	cargo fetch --target $(TARGET)

build: ## Build the release binary
	cargo build --release --target $(TARGET)

run: build ## Build and run the release binary
	./target/$(TARGET)/release/$(BINARY_NAME)

rpm: ## Build an RPM package locally
	./scripts/build-rpm.sh

deb: build ## Build a Debian package locally
	./scripts/build-deb.sh

arch-pkg: ## Build an Arch Linux package locally
	./scripts/build-arch-pkg.sh

alpine-apk: ## Build an Alpine apk package locally
	./scripts/build-alpine-apk.sh

test: ## Run the test suite
	cargo test

release-binary: ## Build a static release binary in a container
	./scripts/build-static-binary-container.sh

release-rpm: ## Build an RPM package in a container
	./scripts/build-rpm-container.sh

release-deb: ## Build a Debian package in a container
	./scripts/build-deb-container.sh

release-arch: ## Build an Arch Linux package in a container
	./scripts/build-arch-pkg-container.sh

release-alpine: ## Build an Alpine apk package in a container
	./scripts/build-alpine-apk-container.sh

install: build ## Install the binary to $(HOME)/bin
	mkdir -p "$(HOME)/bin"
	install -m 0755 "target/$(TARGET)/release/$(BINARY_NAME)" "$(HOME)/bin/"

clean: ## Remove cargo build artifacts
	cargo clean
