BINARY_NAME := sanitize_filenames
TARGET := x86_64-unknown-linux-musl
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)

.PHONY: all build clean deps initialize run test rpm

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
	tar czf $(BINARY_NAME)-$(VERSION).tar.gz \
		--transform='s,^,$(BINARY_NAME)-$(VERSION)/,' \
		--exclude-vcs --exclude target --exclude '*.rpm' --exclude '*.tar.gz' .
	rpmbuild -ba \
		--define "_sourcedir $(PWD)" \
		--define "_srcrpmdir $(PWD)/target/srpm" \
		--define "_rpmdir $(PWD)/target/rpm" \
		--define "_builddir $(PWD)/target/rpmbuild" \
		sanitize_filenames.spec
	rm -f $(BINARY_NAME)-$(VERSION).tar.gz

test:
	cargo test

clean:
	cargo clean
