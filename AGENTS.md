# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace with two crates:
- `crates/web4-core`: compiler/runtime logic, W4ML parsing, shared APIs.
- `crates/web4-gateway`: HTTP gateway binary and rendering/auth/policy handlers.

Supporting folders:
- `docs/`: operational and conformance docs (`RUNBOOK.md`, `CONFORMANCE-MATRIX.md`).
- `examples/`: sample `.w4` documents and service stubs used for local testing.
- `tests/`: reserved top-level test space (currently minimal); most tests live per crate.
- `config.json`: local gateway runtime config.

## Build, Test, and Development Commands
Use workspace-level Cargo commands from the repo root:
- `cargo check --workspace` — fast compile check.
- `cargo build --workspace` — build all crates.
- `cargo test --workspace` — run unit + integration tests.
- `cargo run -p web4-gateway -- config.json` — start the gateway with explicit config.
- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` — format and lint before PRs.

## Coding Style & Naming Conventions
- Follow Rust 2021 defaults; use `rustfmt` output as the style source of truth.
- Use `snake_case` for files, modules, and functions; `PascalCase` for types/traits; `SCREAMING_SNAKE_CASE` for constants.
- Keep modules focused (e.g., gateway logic is split under `src/logic/`).
- Prefer explicit error types (`thiserror`) over stringly typed errors.

## Testing Guidelines
- Do not place tests inside main feature/logic source files; keep tests in `tests/` directories to separate behavior validation from production code.
- Add integration tests under each crate’s `tests/` directory (examples: `runtime_flow.rs`, `path_safety.rs`).
- Keep test names behavior-focused (`rejects_invalid_schema`, `serves_default_template`).
- For parser/runtime changes, include success + failure-path coverage.
- Run `cargo test --workspace` locally before opening a PR.

## Commit & Pull Request Guidelines
Git history is not available in this checkout, so follow Conventional Commits (`feat:`, `fix:`, `chore:`, `test:`) in imperative mood.
For PRs, include:
- scope and motivation,
- linked issue(s) or task reference,
- test evidence (command + result),
- sample request/response or screenshot when gateway behavior changes.

## Security & Configuration Tips
- Never commit real secrets in `config.json` (`jwt_secret`, `admin_token`).
- Keep `rendering.template_loader.allow_remote` disabled unless explicitly needed.
