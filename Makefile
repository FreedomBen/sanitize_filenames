BINARY_NAME := sanitize_filenames
TARGET := x86_64-unknown-linux-musl

.PHONY: all build clean deps initialize run test

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

test:
	cargo test

clean:
	cargo clean
