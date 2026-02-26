# WEB4ORG-STD-0001 Conformance Matrix

> Spec: `WEB4ORG-STD-0001.md` (v0.1, 2026-02-24)
> Scope: Parser / Server Runtime / Agent Runtime (gateway-side implementation)

## Legend

- `DONE`: implemented in current codebase and covered by tests.
- `PLANNED`: mapped, but implementation lands in later phases.

## Clause-level mapping

| Clause | Requirement Summary (`shall`) | Target | Code Mapping | Test Mapping | Status |
|---|---|---|---|---|---|
| 5.2 | W4ML document conformance checks | Parser | `crates/web4-core/src/w4ml.rs` | `crates/web4-core/src/w4ml.rs:654` | DONE |
| 5.3 | Parser conformance + recovery/fatal rules | Parser | `crates/web4-core/src/w4ml.rs` | `crates/web4-core/src/w4ml.rs:666` | DONE |
| 5.4 | Server runtime conformance (negotiation/consent/policy) | Server Runtime | `crates/web4-gateway/src/main.rs`, `crates/web4-core/src/runtime.rs` | `crates/web4-gateway/src/main.rs:1104`, `crates/web4-core/src/runtime.rs:597` | DONE |
| 5.5 | Agent runtime conformance hooks | Agent Runtime | `crates/web4-core/src/traits.rs` | `tests/` (to add in Phase E/F) | PLANNED |
| 8.3 | Parse error recovery and fatal error behavior | Parser | `crates/web4-core/src/w4ml.rs` | `crates/web4-core/src/w4ml.rs:679` | DONE |
| 12 | Effects enforcement before invocation | Server Runtime | `crates/web4-core/src/runtime.rs` | `crates/web4-core/src/runtime.rs:618` | DONE |
| 13 | Consent mechanism enforcement | Server Runtime / Agent Runtime | `crates/web4-gateway/src/main.rs`, `crates/web4-core/src/runtime.rs` | `crates/web4-gateway/src/main.rs:1034`, `crates/web4-core/src/runtime.rs:622` | DONE |
| 15.2 | HTTP binding input/output invocation semantics | Server Runtime | `crates/web4-core/src/runtime.rs` | `crates/web4-core/src/runtime.rs:650` | DONE |
| 15.3 | Local binding isolation and runtime entrypoint form | Server Runtime | `crates/web4-core/src/runtime.rs` | `crates/web4-core/src/runtime.rs:694` | DONE |
| 16.1 | Runtime error JSON body structure | Server Runtime | `crates/web4-core/src/error.rs` | `crates/web4-core/src/error.rs:138` | DONE |
| 16.2 | Standard error code -> HTTP status mapping | Server Runtime | `crates/web4-core/src/error.rs` | `crates/web4-core/src/error.rs:142`, `crates/web4-gateway/src/main.rs:58` | DONE |
| 18 | Content negotiation + `Vary: Accept` | Server Runtime | `crates/web4-gateway/src/main.rs` | `crates/web4-gateway/src/main.rs:584` | DONE |
| 20.2 | `w4fragment` selector semantics/errors | Server Runtime | `crates/web4-gateway/src/main.rs` | `crates/web4-gateway/src/main.rs:603` | DONE |
| 22.1 | Parser can parse+validate for compilation pipeline | Parser | `crates/web4-core/src/w4ml.rs` | `crates/web4-core/src/w4ml.rs:38` | DONE |
| 22.2 | Compile to OpenAI-like tool JSON + `x-w4` | Compiler | `crates/web4-core/src/compiler.rs` | `crates/web4-core/src/compiler.rs:16` | DONE |
| 24.1 | Human rendering contract | Server Runtime | `crates/web4-gateway/src/main.rs` | `crates/web4-gateway/src/main.rs:1583` | DONE |

## Phase A-H completion notes

- Implemented W4ML parser and semantic validator with fatal parse diagnostics and Clause 8.3 recovery coverage.
- Added normalized parser output and index model (`section_ids`, `service_ids`, `type_ids`).
- Enforced schema uniqueness, id constraints, section/service uniqueness, and `typeRef` resolvability.
- Delivered content negotiation (`application/w4ml+xml` / `text/html` / `application/w4+json`) with `Vary: Accept` and `w4fragment` selector support.
- Added service execution pipeline: input/output JSON Schema validation, effects/consent gate, and binding execution.
- Implemented HTTP + local (`bin:`/`mcp:`) binding executors with standardized Clause 16 error responses.
- Added fixed JWT capability validation (`iss/sub/exp/scope` + signature), issue/present flow, and interactive challenge approve/deny lifecycle with invocation binding.
- Added policy and security runtime controls: `rateLimit` (429/`RATE_LIMITED`), `allowOrigins` CORS mapping, `allowAgents` identity gate, and payload policy-override stripping for injection boundary.
- Added human rendering template engine flow: built-in default template, external `link rel="template"` support, class-based hooks, and lazy/never folded-or-omitted first-screen behavior with body-order preservation.
- Completed interoperability and release baseline: `application/w4+json` includes `imports`/`peers`, MCP direct example docs, binary JSON-file startup config, and stable library runtime builder API.
- Kept parser/compiler/gateway baselines stable; workspace test suite remains green.
