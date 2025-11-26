# Agent Instructions for `sanitize_filenames`

- Scope: applies to the entire `sanitize_filenames` repository.
- When running build or test commands (e.g., `cargo build`, `cargo test`, `make`, or `make test`), first ensure that the `PATH` **does not** include any linuxbrew-related directories (any segment containing `linuxbrew`). This avoids linker/GLIBC issues from picking up linuxbrew toolchains instead of the system toolchain.

