pkgname=sanitize-filenames
pkgver=0.2.0
pkgrel=1
pkgdesc='CLI tool to sanitize filenames'
arch=('x86_64')
url='https://example.com/sanitize_filenames'
license=('AGPL3')
depends=()
makedepends=('cargo' 'rust' 'musl')
source=()
sha256sums=()

build() {
  cd "$srcdir/.."
  cargo build --release --target x86_64-unknown-linux-musl
}

package() {
  cd "$srcdir/.."

  install -Dm755 "target/x86_64-unknown-linux-musl/release/sanitize_filenames" \
    "$pkgdir/usr/bin/sanitize_filenames"

  install -Dm644 "LICENSE" \
    "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

  install -Dm644 "README.md" \
    "$pkgdir/usr/share/doc/$pkgname/README.md"

  install -Dm644 "man/sanitize_filenames.1" \
    "$pkgdir/usr/share/man/man1/sanitize_filenames.1"
}
