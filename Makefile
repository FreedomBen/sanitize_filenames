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
	tar czf $(BINARY_NAME)-$(VERSION).tar.gz \
		--transform='s,^,$(BINARY_NAME)-$(VERSION)/,' \
		--exclude-vcs --exclude target --exclude '*.rpm' --exclude '*.tar.gz' . || [ $$? -eq 1 ]
	rpmbuild -ba \
		--define "_sourcedir $(PWD)" \
		--define "_srcrpmdir $(PWD)/target/srpm" \
		--define "_rpmdir $(PWD)/target/rpm" \
		--define "_builddir $(PWD)/target/rpmbuild" \
		sanitize_filenames.spec
	rm -f $(BINARY_NAME)-$(VERSION).tar.gz

deb: build
	@mkdir -p target/deb
	@rm -rf target/deb/$(DEB_PACKAGE)_$(VERSION)
	mkdir -p target/deb/$(DEB_PACKAGE)_$(VERSION)/DEBIAN
	mkdir -p target/deb/$(DEB_PACKAGE)_$(VERSION)/usr/bin
	mkdir -p target/deb/$(DEB_PACKAGE)_$(VERSION)/usr/share/doc/$(DEB_PACKAGE)
	install -m 0755 target/$(TARGET)/release/$(BINARY_NAME) \
		target/deb/$(DEB_PACKAGE)_$(VERSION)/usr/bin/$(BINARY_NAME)
	cp LICENSE README.md \
		target/deb/$(DEB_PACKAGE)_$(VERSION)/usr/share/doc/$(DEB_PACKAGE)/
	printf "Package: $(DEB_PACKAGE)\n" > target/deb/$(DEB_PACKAGE)_$(VERSION)/DEBIAN/control
	printf "Version: $(VERSION)-1\n" >> target/deb/$(DEB_PACKAGE)_$(VERSION)/DEBIAN/control
	printf "Section: utils\n" >> target/deb/$(DEB_PACKAGE)_$(VERSION)/DEBIAN/control
	printf "Priority: optional\n" >> target/deb/$(DEB_PACKAGE)_$(VERSION)/DEBIAN/control
	printf "Architecture: $(DEB_ARCH)\n" >> target/deb/$(DEB_PACKAGE)_$(VERSION)/DEBIAN/control
	printf "Maintainer: Benjamin Porter <freedomben@protonmail.com>\n" >> target/deb/$(DEB_PACKAGE)_$(VERSION)/DEBIAN/control
	printf "Description: CLI tool to sanitize filenames\n" >> target/deb/$(DEB_PACKAGE)_$(VERSION)/DEBIAN/control
	dpkg-deb --build target/deb/$(DEB_PACKAGE)_$(VERSION) \
		target/deb/$(DEB_PACKAGE)_$(VERSION)_$(DEB_ARCH).deb

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
