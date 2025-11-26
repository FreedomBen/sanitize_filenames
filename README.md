# sanitize_filenames

Rust implementation of a filename sanitizer that builds to a statically linked binary.

## Prerequisites

- Rust toolchain (via `rustup`): https://rustup.rs
- `musl` target for Rust:
  - `rustup target add x86_64-unknown-linux-musl`
- `musl-gcc` installed on your system (used as the linker for static builds).  
  On many Linux distros this is provided by a package like:
  - Debian/Ubuntu: `sudo apt-get install musl-tools`
  - Fedora: `sudo dnf install musl-gcc`

### Fedora setup

On Fedora, you can install all required system packages (including `rustup`) with:

```sh
sudo dnf install rustup musl-gcc make gcc
```

Then initialize Rust and install the musl target:

```sh
rustup-init
source ~/.cargo/env   # or start a new shell
rustup target add x86_64-unknown-linux-musl
```

## Quick start (first-time setup)

From the project root, after installing Rust and `musl-gcc`:

```sh
rustup target add x86_64-unknown-linux-musl
make deps   # optional but recommended
make        # or: make build
```

If you see an error like:

```text
error[E0463]: can't find crate for `std`
  = note: the `x86_64-unknown-linux-musl` target may not be installed
```

it means the musl target is missing; fix it with:

```sh
rustup target add x86_64-unknown-linux-musl
```

## Project layout

- `Cargo.toml` – Rust package manifest.
- `src/main.rs` – Binary entrypoint (currently prints `working`).
- `.cargo/config.toml` – Configures the default build target to `x86_64-unknown-linux-musl` and uses `musl-gcc` as the linker.
- `Makefile` – Convenience targets for building, fetching dependencies, and cleaning.

## Building the binary

You can build using `make` (recommended) or `cargo` directly.

### Using Make

From the project root:

```sh
make deps     # optional: fetch all Cargo dependencies
make          # or: make build
```

This will produce a statically linked release binary under:

```sh
target/x86_64-unknown-linux-musl/release/sanitize_filenames
```

### Using Cargo directly

From the project root:

```sh
cargo build --release --target x86_64-unknown-linux-musl
```

The resulting binary path is the same as above.

## Cleaning build artifacts

To remove all build artifacts (including `target/`), run:

```sh
make clean
```

or, equivalently:

```sh
cargo clean
```

## Running the binary

After building:

```sh
./target/x86_64-unknown-linux-musl/release/sanitize_filenames
```

You should see:

```text
working
```
