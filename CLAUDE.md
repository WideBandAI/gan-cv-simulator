# Development Guidelines (Rust)

This document contains critical information about working with this codebase. Follow these guidelines precisely.

## Core Development Rules

1. Package Management
   - ONLY use cargo
   - Installation: `cargo add crate`
   - Running tools: `cargo run`
   - Testing: `cargo test`
   - Upgrading: `cargo update`
   - FORBIDDEN: manual editing of Cargo.lock

2. Code Quality
   - Explicit types preferred for public APIs
   - Public APIs must have documentation comments (`///`)
   - Functions must be focused and small
   - Follow existing patterns exactly
   - Line length: 100 chars maximum (rustfmt default)

3. Testing Requirements
   - Framework: `cargo test`
   - Use `#[test]` for unit tests
   - Integration tests: `tests/` directory
   - Coverage: test edge cases and errors
   - New features require tests
   - Bug fixes require regression tests

4. Code Style
   - Naming:
     - snake_case for functions/variables
     - PascalCase for structs/enums
     - UPPER_SNAKE_CASE for constants
   - Use `rustfmt` formatting
   - Prefer `Result<T, E>` over panics
   - Avoid `unwrap()` in production code
   - Use `?` operator for error propagation

5. Pre-commit (optional but strongly recommended)
   - Before every commit, you MUST run:
     ```bash
     cargo fmt --all
     cargo clippy --all -- -D warnings
     ```

   - Recommended full check sequence:
     ```bash
     cargo fmt --all
     cargo clippy --all -- -D warnings
     cargo test
     ```

   - Do not commit if:
     - Formatting is not clean
     - Clippy reports warnings (treated as errors)
     - Tests are failing

## Commit Message Guidelines

- For commits fixing bugs or adding features based on user reports add:
  ```bash
  git commit --trailer "Reported-by: <name>"