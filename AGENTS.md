# Agent Instructions for `sanitize_filenames`

- Scope: applies to the entire `sanitize_filenames` repository.
- When making any changes that affect the shell autocomplete functionality, be sure to update the autocomplete functionality
- Write comprehensive tests when changing functionality that will exercise the expected output with a number of different test cases to ensure there are no unexpected edge cases that perform incorrectly
- When running build or test commands (e.g., `cargo build`, `cargo test`, `make`, or `make test`), first ensure that the `PATH` **does not** include any linuxbrew-related directories (any segment containing `linuxbrew`). This avoids linker/GLIBC issues from picking up linuxbrew toolchains instead of the system toolchain.

