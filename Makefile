BINARY_NAME := sanitize_filenames
TARGET := x86_64-unknown-linux-musl
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)
DEB_PACKAGE := sanitize-filenames
DEB_ARCH := amd64

.PHONY: all build clean deps initialize run test rpm deb \
	release-binary release-rpm release-deb

all: build

initialize:
	rustup target add $(TARGET)
	$(MAKE) deps
	$(MAKE) build

deps:
	cargo fetch --target $(TARGET)

build:
	cargo build --release --target $(TARGET)

run: build
	./target/$(TARGET)/release/$(BINARY_NAME)

rpm:
	./scripts/build-rpm.sh

deb: build
	./scripts/build-deb.sh

test:
	cargo test

release-binary:
	./scripts/build-static-binary-container.sh

release-rpm:
	./scripts/build-rpm-container.sh

release-deb:
	./scripts/build-deb-container.sh

clean:
	cargo clean
