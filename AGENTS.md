# Repository Guidelines

## Project Structure & Module Organization
- Workspace root hosts the `ros2_message` crate (library) and `examples` crate (runnable demos). Shared config lives in the top-level `Cargo.toml`.
- Library sources: `ros2_message/src/*`; parsing helpers in `src/parse_msg`, and core types in `data_type.rs`, `msg.rs`, `srv.rs`, and `dynamic/`.
- Tests: unit tests colocated under `ros2_message/src/tests` and `src/parse_msg/tests.rs`. Example binaries live in `examples/examples/*.rs`; extra sample assets under `msg_examples/`.

## Build, Test, and Development Commands
- Format: `cargo fmt --all` (run before commits).
- Lint: `cargo clippy --workspace --all-targets --all-features` (fail on warnings).
- Build: `cargo build --workspace --all-features` for full feature coverage (`decode` + `mcap` are default).
- Tests: `cargo test --workspace` for the full suite; target a crate with `cargo test -p ros2_message`.
- Examples: `cargo run -p examples --example mcap_reading -- <path-to.mcap>` (see other examples in `examples/examples/`).

## Coding Style & Naming Conventions
- Rust 2021; follow `cargo fmt` defaults (4-space indent, trailing commas preferred).
- Modules and files use `snake_case`; public types and traits use `CamelCase`; constants use `SCREAMING_SNAKE_CASE`.
- Prefer explicit feature gating (`decode`, `mcap`) and keep error handling via `Result`/`thiserror` consistent.
- Keep docs/comments focused on behavior and ROS2 nuances rather than restating signatures.

## Testing Guidelines
- Rely on `cargo test`; no custom harness. Add focused unit tests near the code under `src/tests` to cover new ROS2 encodings, edge cases, and regression fixes.
- When decoding/encoding behavior changes, include round-trip or fixture-based tests (e.g., using assets from `msg_examples/`).
- Aim to keep CI-friendly tests: avoid network/file-system dependencies outside provided fixtures.

## Commit & Pull Request Guidelines
- Commit messages mirror the existing short imperative style (`fix TF decoding`, `clean code`). Use small, focused commits.
- Pull requests should describe scope, highlight feature flags touched, and mention impacted examples/tests. Link issues when applicable and include repro or sample commands.
- Include before/after notes or screenshots only when changes affect example outputs or observable behavior.
