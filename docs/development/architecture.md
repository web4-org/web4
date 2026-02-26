# Architecture

This document describes the internal architecture of the web4 implementation, tracing data
from a W4ML file on disk to an HTTP response.

---

## High-Level Pipeline

```
.w4 file (disk)
    │
    ▼
W4mlParser::parse()
    │ ParseResult (normalized internal model)
    ▼
┌─────────────────────────────────────────────────────┐
│                    AppState                         │
│  ┌────────────┐   ┌──────────────────────────────┐  │
│  │ Arc<Model> │   │ DefaultServiceRuntime         │  │
│  │            │   │  ├─ JsonSchemaValidator       │  │
│  │            │   │  ├─ DefaultConsentEngine      │  │
│  │            │   │  └─ GatewayBindingExecutor    │  │
│  └────────────┘   │      ├─ HttpBindingExecutor   │  │
│                   │      └─ LocalBindingExecutor  │  │
│  ┌────────────┐   └──────────────────────────────┘  │
│  │ RateLimiter│                                      │
│  │ ChallengeStore                                   │
│  └────────────┘                                      │
└─────────────────────────────────────────────────────┘
    │
    ▼
Axum Router (14 routes)
    │
    ├─ GET /           → get_document() → content negotiation → response
    ├─ GET /{ref}      → view_document_by_path()
    ├─ POST /{ref}     → invoke_service_by_source_ref()
    ├─ POST /consent/issue         → issue_capability()
    ├─ POST /consent/challenge     → create_challenge()
    ├─ POST /consent/challenge/{id}/approve → approve_challenge()
    ├─ POST /consent/challenge/{id}/deny    → deny_challenge()
    └─ GET /healthz    → healthz()
```

---

## `web4-core` Crate

### Parser (`w4ml.rs`)

The entry point is `W4mlParser::parse(source: &str)`. It:

1. Parses the XML structure using `roxmltree`.
2. Validates document conformance (namespace, version, required elements).
3. Resolves `typeRef` references, checking that every reference points to a declared type.
4. Validates uniqueness constraints (section IDs, service IDs, type IDs).
5. Returns a `ParseResult` containing the normalized model or a list of fatal diagnostics.

**Key types:**
- `ParseResult` — contains the normalized `W4Document` or parse errors
- `W4Document` — sections, services, schema types, metadata, links
- `ServiceDeclaration` — all fields from the `<service>` element, resolved

### Compiler (`compiler.rs`)

`compile_to_tool_json(model: &W4Document) -> Vec<ToolDefinition>`

Converts each service in the normalized model to an OpenAI-like tool JSON object:

1. Resolves the service's input and output `typeRef` to `W4Schema` types.
2. Converts `W4Schema` to JSON Schema objects.
3. Adds `x-w4` extension metadata (load, class, invoke URL, consent mode, effects).

### Runtime (`runtime/mod.rs`)

`ServiceRuntime<V, C, B>` is the generic service execution engine, parameterized over:

- `V: Validator` — validates input/output JSON against schemas
- `C: ConsentEngine` — enforces consent mode gates
- `B: BindingExecutor` — dispatches to the correct transport

The concrete type used by the gateway is:

```rust
type DefaultServiceRuntime =
    ServiceRuntime<JsonSchemaValidator, DefaultConsentEngine, GatewayBindingExecutor>;
```

**Invocation flow in `ServiceRuntime::invoke`:**

```
invoke(service_id, input, consent_context)
  1. effects_check(service)
  2. consent_engine.check(ConsentRequest { mode, context })
  3. validator.validate(input, service.input_schema)
  4. binding_executor.execute(BindingInvocation { service, input })
  5. validator.validate(output, service.output_schema)
  6. return output
```

### Consent Engine (`runtime/consent.rs`)

`DefaultConsentEngine::check()` implements three paths:

- **Open:** returns `Ok(())` immediately.
- **Capability:** extracts the JWT from the context, verifies HMAC-SHA256 signature, checks
  `iss`, `sub`, `exp`, and `scope` claims.
- **Interactive:** looks up the challenge ID in the challenge store, verifies `approved` status,
  consumes the challenge.

### Binding Executors (`runtime/binding.rs`)

`GatewayBindingExecutor` delegates to:

- `HttpBindingExecutor` — sends a `reqwest` POST to the resolved endpoint, reads the JSON response.
- `LocalBindingExecutor` — spawns the process via `tokio::process::Command`, pipes JSON to stdin,
  reads stdout.

### Schema Validation (`runtime/validation.rs`, `runtime/schema.rs`)

`JsonSchemaValidator` converts W4Schema types to JSON Schema objects and validates using the
`jsonschema` crate. Validation is strict — extra properties are rejected when
`additionalProperties: false` is declared.

### Error Types (`error.rs`)

`ErrorCode` is the canonical error vocabulary. Each variant maps to an HTTP status code.
`RuntimeError` is the wire format (code + message + retryable + optional details).
`ErrorEnvelope` wraps `RuntimeError` in the `{ "error": ... }` envelope.

---

## `web4-gateway` Crate

### App Initialization (`app.rs`)

`build_state()`:
1. Canonicalizes the document root path.
2. Parses the entry `.w4` file.
3. Builds the `DefaultServiceRuntime`.
4. Initializes `InMemoryRateLimiter` and `InMemoryChallengeStore`.
5. Returns `AppState` (wrapped in `Arc` for shared ownership).

`build_app()`: constructs the Axum `Router` with all routes and attaches `AppState`.

### Handlers (`handlers.rs`)

Each handler function:
1. Extracts path parameters, query parameters, headers, and JSON body using Axum extractors.
2. Delegates to the relevant `logic/` module functions.
3. Returns a typed response or a `GatewayError`.

### Logic Layer (`logic/`)

Thin, testable business logic separated from Axum:

| Module | Responsibility |
|---|---|
| `auth.rs` | JWT extraction and validation |
| `content.rs` | Content negotiation, W4+JSON compilation |
| `consent.rs` | Challenge creation, approval, consumption |
| `policy.rs` | Rate limiting, CORS, agent allowlist, payload stripping |
| `common.rs` | Service lookup, fragment selection |

### Renderer (`renderer.rs`)

`render_html(doc, options)`:
1. Resolves the template (document-linked or built-in default).
2. Builds the `page` context object from the normalized model.
3. Renders with `tera::Tera`.
4. Returns the HTML string.

Template loading respects `allow_remote`, `allowed_remote_hosts`, `timeout_ms`, and `max_bytes`
from config.

### State (`state.rs`)

`AppState` is `Clone + Send + Sync` (backed by `Arc`). It holds:

- `Arc<W4Document>` — the parsed, immutable document model
- `DefaultServiceRuntime` — the service execution engine
- `InMemoryRateLimiter` — per-service sliding-window rate limiter (uses `AtomicU64` counters)
- `InMemoryChallengeStore` — UUID-keyed challenge records with status and expiry

---

## Extension Points

The `web4-core` crate exposes Rust traits for replacing any layer:

```rust
pub trait Validator: Send + Sync {
    fn validate(&self, target: &ValidationTarget) -> Result<(), Web4Error>;
}

pub trait ConsentEngine: Send + Sync {
    async fn check(&self, request: &ConsentRequest) -> Result<(), Web4Error>;
}

pub trait BindingExecutor: Send + Sync {
    async fn execute(&self, invocation: &BindingInvocation) -> Result<Value, Web4Error>;
}

pub trait Renderer: Send + Sync {
    fn render_html(&self, request: &RenderRequest) -> Result<String, Web4Error>;
}
```

Implement any of these traits and pass them to `ServiceRuntime<V, C, B>` to customize the
runtime behaviour without modifying the gateway.

See [Library API](library-api.md) for how to embed `web4-core` in your own application.
