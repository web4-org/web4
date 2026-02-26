# Testing

This guide covers how to run the existing test suite and how to write new tests.

---

## Running Tests

### Full workspace

```bash
cargo test --workspace
```

### Single crate

```bash
cargo test -p web4-core
cargo test -p web4-gateway
```

### Single test file

```bash
cargo test -p web4-core --test w4ml_parser
```

### Single test function

```bash
cargo test -p web4-gateway --test gateway_integration -- rate_limit_returns_429
```

### With output (for debugging)

```bash
cargo test --workspace -- --nocapture
```

---

## Test Layout

Tests are organized per crate in `tests/` directories (not inside source files):

```
crates/web4-core/tests/
├── w4ml_parser.rs          W4ML parsing success and failure paths
├── compiler_contract.rs    Compiler output contract tests
├── runtime_flow.rs         End-to-end runtime invocation flows
├── error_status.rs         Error code → HTTP status mapping
└── api_runtime.rs          Public API (RuntimeOptions, build_default_runtime)

crates/web4-gateway/tests/
├── gateway_integration.rs  Full HTTP request/response integration tests
├── config_loading.rs       Config file loading and validation
├── path_safety.rs          Path traversal rejection tests
└── renderer_templates.rs   HTML template rendering tests
```

---

## Writing a Unit Test

Add a new file to the relevant crate's `tests/` directory. Do not add `#[cfg(test)]` blocks
inside source files.

```rust
// crates/web4-core/tests/my_feature.rs

use web4_core::W4mlParser;

#[test]
fn parses_minimal_valid_document() {
    let source = r#"<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="https://example.com/test.w4">
  <head><title lang="en">Test</title></head>
  <body></body>
</w4>"#;

    let result = W4mlParser::parse(source);
    assert!(result.is_ok(), "expected parse success, got: {result:?}");
}

#[test]
fn rejects_missing_namespace() {
    let source = r#"<?xml version="1.0" encoding="UTF-8"?>
<w4 version="0.1" id="https://example.com/test.w4">
  <head><title>Test</title></head>
  <body></body>
</w4>"#;

    let result = W4mlParser::parse(source);
    assert!(result.is_err(), "expected parse failure for missing namespace");
}
```

---

## Writing a Gateway Integration Test

Use Axum's `TestClient` or `tower::ServiceExt` to test the full HTTP stack:

```rust
// crates/web4-gateway/tests/my_endpoint.rs

use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use web4_gateway::app::build_app;

fn build_test_state() -> AppState {
    // ... construct a minimal AppState with a test document
}

#[tokio::test]
async fn healthz_returns_ok() {
    let app = build_app(build_test_state(), false);

    let response = app
        .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## Test Naming Guidelines

Name tests by the behaviour they verify, not the function they call:

| Bad name | Good name |
|---|---|
| `test_parser` | `parses_schema_with_multiple_types` |
| `test_handler_1` | `invoke_open_service_returns_output` |
| `test_error` | `rate_limit_returns_429_with_retryable_true` |

Pattern: `verb_subject_condition_outcome`

- `rejects_invalid_schema`
- `serves_default_template_when_no_template_declared`
- `consent_required_returned_for_capability_service_without_token`

---

## Coverage Guidelines

For every new feature, provide:

1. **Happy path** — the feature works as specified
2. **Failure path** — invalid input, missing auth, policy violation, etc.
3. **Edge cases** — empty inputs, boundary values, duplicate IDs, etc.

For parser/compiler changes, test against both valid and invalid W4ML documents.

---

## Pre-PR Checklist

```bash
# Must all pass before opening a PR
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
