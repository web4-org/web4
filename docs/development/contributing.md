# Contributing

This document consolidates all guidelines for contributing to the web4 repository.

---

## Repository Structure

This is a Rust workspace with two crates:

| Crate | Path | Purpose |
|---|---|---|
| `web4-core` | `crates/web4-core/` | W4ML parser, compiler, runtime library, shared types |
| `web4-gateway` | `crates/web4-gateway/` | HTTP gateway binary, rendering, auth, policy handlers |

Supporting directories:

| Directory | Contents |
|---|---|
| `docs/` | User-facing documentation |
| `examples/` | Sample `.w4` documents and service stubs for local testing |
| `tests/` | Reserved top-level test space; most tests live per-crate |

---

## Build and Test Commands

Run all commands from the repository root:

```bash
# Fast compile check (no binaries)
cargo check --workspace

# Full build
cargo build --workspace

# Run all tests
cargo test --workspace

# Start gateway with showcase config
cargo run -p web4-gateway -- config.showcase.json

# Format code
cargo fmt --all

# Lint (must pass before PR)
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Coding Style

- **Language:** Rust 2021 edition.
- **Formatting:** `rustfmt` is the source of truth. Run `cargo fmt --all` before committing.
- **Naming:**
  - `snake_case` — files, modules, functions, variables
  - `PascalCase` — types, traits, enums
  - `SCREAMING_SNAKE_CASE` — constants
- **Error types:** Prefer `thiserror`-derived error types over stringly typed errors.
- **Module focus:** Keep modules single-purpose. Gateway logic is split under `src/logic/`
  (`auth`, `content`, `consent`, `policy`, `common`).

---

## Testing Guidelines

- Do not place tests inside main feature source files. Keep tests in `tests/` directories.
- Add integration tests under each crate's `tests/` directory:
  - `crates/web4-core/tests/` — parser, compiler, runtime flow, error mapping, API
  - `crates/web4-gateway/tests/` — gateway integration, config loading, path safety, rendering
- Name tests to describe behaviour, not implementation:
  - Good: `rejects_invalid_schema`, `serves_default_template`, `rate_limit_returns_429`
  - Bad: `test_validator`, `test_handler_1`
- For parser/runtime changes, include both success-path and failure-path coverage.
- Run `cargo test --workspace` locally before opening a PR.

---

## Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/) with imperative mood:

| Type | Use for |
|---|---|
| `feat:` | New functionality |
| `fix:` | Bug fix |
| `chore:` | Build, CI, dependency updates |
| `test:` | Adding or fixing tests |
| `docs:` | Documentation changes |
| `refactor:` | Code restructuring without behaviour change |

Examples:

```
feat: add rate-limit enforcement to service invocation pipeline
fix: correct JWT scope check for multi-service documents
test: add path traversal rejection tests to gateway integration suite
docs: add agent integration guide
```

---

## Pull Request Checklist

Before opening a PR, verify:

- [ ] `cargo fmt --all` — no formatting diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- [ ] `cargo test --workspace` — all tests pass
- [ ] New behaviour is covered by at least one test
- [ ] PR description includes:
  - **Scope and motivation** — what does this change and why
  - **Linked issue(s)** — reference any relevant issue or task
  - **Test evidence** — paste the test command and its output
  - **Sample request/response** — for any gateway behaviour changes

---

## Security Contributions

If you discover a security vulnerability, do not open a public issue. Follow responsible
disclosure:

1. Email the maintainers directly (see repository metadata for contact).
2. Include a description of the issue, reproduction steps, and potential impact.
3. Allow reasonable time for a fix before public disclosure.
