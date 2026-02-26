# Library API

`web4-core` is a stable Rust library that can be embedded in your own application without
taking a dependency on the gateway binary or Axum.

---

## Adding the Dependency

In your `Cargo.toml`:

```toml
[dependencies]
web4-core = { path = "../web4/crates/web4-core" }
# or, when published to crates.io:
# web4-core = "0.1"
```

---

## Stable Public API

The stable public surface is defined in `crates/web4-core/src/api.rs`:

### `RuntimeOptions`

```rust
pub struct RuntimeOptions {
    pub http_base_url: Option<String>,
    pub local_working_dir: Option<String>,
}
```

| Field | Type | Description |
|---|---|---|
| `http_base_url` | `Option<String>` | Base URL for resolving relative HTTP binding endpoints |
| `local_working_dir` | `Option<String>` | Working directory for local binding process execution |

`RuntimeOptions` implements `Default`:

```rust
let options = RuntimeOptions::default();
// http_base_url: None, local_working_dir: None
```

### `build_default_runtime`

```rust
pub fn build_default_runtime(options: RuntimeOptions) -> DefaultServiceRuntime
```

Constructs a `DefaultServiceRuntime` wired with:
- `JsonSchemaValidator` — strict JSON Schema validation
- `DefaultConsentEngine` — enforces open/capability/interactive consent
- `GatewayBindingExecutor` — dispatches to HTTP and Local binding executors

### `DefaultServiceRuntime`

```rust
pub type DefaultServiceRuntime =
    ServiceRuntime<JsonSchemaValidator, DefaultConsentEngine, GatewayBindingExecutor>;
```

The concrete runtime type. Pass it to your application's invocation logic.

---

## Parsing a W4ML Document

```rust
use web4_core::W4mlParser;

let source = std::fs::read_to_string("my-site/index.w4")?;
let parse_result = W4mlParser::parse(&source)?;
let document = parse_result.document; // W4Document
```

`W4mlParser::parse` returns a `Result<ParseResult, Web4Error>`. Fatal parse errors are
returned as `Err(...)`. Non-fatal diagnostics are available in `parse_result.diagnostics`.

---

## Compiling to Tool JSON

```rust
use web4_core::compile_to_tool_json;

let tools = compile_to_tool_json(&document);
let json = serde_json::to_string_pretty(&tools)?;
println!("{json}");
```

Each element in `tools` is an OpenAI-like tool definition with an `x-w4` extension block.

---

## Invoking a Service

```rust
use web4_core::{build_default_runtime, RuntimeOptions};
use serde_json::json;

let options = RuntimeOptions {
    http_base_url: Some("http://127.0.0.1:8080".into()),
    local_working_dir: Some("/path/to/document/root".into()),
};
let runtime = build_default_runtime(options);

let input = json!({"a": 3, "b": 4});
let output = runtime.invoke("math.add", &input, &consent_context).await?;
println!("{output}");
```

`invoke` takes:
- `service_id: &str` — the service ID as declared in the W4ML document
- `input: &Value` — the JSON input payload
- `consent_context: &ConsentContext` — carries the JWT token or challenge ID for consent enforcement

---

## Custom Implementations

Replace any layer by implementing the corresponding trait:

### Custom Validator

```rust
use web4_core::{Validator, ValidationTarget, Web4Error};

struct StrictValidator;

impl Validator for StrictValidator {
    fn validate(&self, target: &ValidationTarget) -> Result<(), Web4Error> {
        // your validation logic
        Ok(())
    }
}
```

### Custom Binding Executor

```rust
use web4_core::{BindingExecutor, BindingInvocation, Web4Error};
use async_trait::async_trait;
use serde_json::Value;

struct MyExecutor;

#[async_trait]
impl BindingExecutor for MyExecutor {
    async fn execute(&self, invocation: &BindingInvocation) -> Result<Value, Web4Error> {
        // your execution logic
        todo!()
    }
}
```

### Wiring Custom Components

```rust
use web4_core::runtime::ServiceRuntime;

let runtime = ServiceRuntime {
    validator: StrictValidator,
    consent_engine: DefaultConsentEngine,
    binding_executor: MyExecutor,
};
```

---

## Error Handling

All library functions return `Result<_, Web4Error>`. The error type:

```rust
pub enum Web4Error {
    Runtime(RuntimeError),
    Internal(String),
}
```

`RuntimeError` carries a typed `ErrorCode`, a human-readable message, and a `retryable` flag.
Convert to an HTTP status code with `error.status_code()`.

```rust
match runtime.invoke(...).await {
    Ok(output) => { /* use output */ }
    Err(e) => {
        let status = e.status_code();
        let envelope = e.into_envelope();
        // serialize envelope as JSON error response
    }
}
```

---

## Complete Embedded Example

```rust
use web4_core::{W4mlParser, build_default_runtime, RuntimeOptions};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse document
    let source = std::fs::read_to_string("my-site/index.w4")?;
    let parse_result = W4mlParser::parse(&source)?;

    // Build runtime
    let runtime = build_default_runtime(RuntimeOptions {
        local_working_dir: Some("my-site".into()),
        ..Default::default()
    });

    // Invoke a service
    let output = runtime.invoke(
        "greet",
        &json!({"name": "Alice"}),
        &Default::default(), // open consent — no token needed
    ).await?;

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
```
