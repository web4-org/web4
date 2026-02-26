# Bindings

A **binding** specifies the concrete execution mapping for a service: the transport protocol,
endpoint or entrypoint, and invocation parameters. Bindings are declared in `.w4s` files.

There are three binding types: `http`, `local`, and `gateway`.

---

## HTTP Binding

Routes service invocations to an external HTTP endpoint.

```xml
<w4s xmlns="urn:w4ml:0.1" service="my.service">
  <bindings>
    <binding type="http"
             method="POST"
             endpoint="https://api.example.com/tools/compute"
             contentType="application/json"/>
  </bindings>
</w4s>
```

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `type` | yes | `http` |
| `method` | yes | HTTP method (`GET`, `POST`, etc.) |
| `endpoint` | yes | Full URL or gateway-relative path |
| `contentType` | no | Request `Content-Type` (default: `application/json`) |

**Runtime behaviour:**

1. The gateway serializes the validated input as JSON.
2. Issues the HTTP request to `endpoint` with the configured `method` and `contentType`.
3. Deserializes and validates the response against the declared output schema.
4. Returns the validated output to the caller.

If `endpoint` is a relative path (e.g., `/services/other.w4s`), it is resolved against the
gateway's `runtime.http_base_url` from the config. This allows services to chain to each other
internally (the capability bridge pattern in the showcase).

---

## Local Binding

Executes a local process or binary on the gateway host.

```xml
<w4s xmlns="urn:w4ml:0.1" service="my.service">
  <bindings>
    <binding type="local" exec="bin:python3 services/my_script.py"/>
  </bindings>
</w4s>
```

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `type` | yes | `local` |
| `exec` | yes | Execution descriptor (see formats below) |

**Exec descriptor formats:**

| Prefix | Example | Description |
|---|---|---|
| `bin:` | `bin:python3 services/calc.py open` | Run a binary/script; args follow the path |
| `mcp:` | `mcp:my-server` | Delegate to an MCP (Model Context Protocol) server |

**Runtime behaviour:**

1. The gateway serializes the validated input as JSON and passes it to the process via `stdin`.
2. The process runs with the `local_working_dir` (configured in `RuntimeOptions`) as the working
   directory.
3. The process writes JSON to `stdout`.
4. The gateway reads `stdout`, deserializes, and validates against the output schema.
5. Stderr is captured for logging.

**Security note:** Local bindings execute on the gateway host. Validate all inputs strictly and
follow least-privilege principles for the process. See [Securing Services](../guides/securing-services.md).

**Example script contract (Python):**

```python
import json, sys

payload = json.load(sys.stdin)
result = {
    "sum": payload["a"] + payload["b"],
    "narrative": f"Computed {payload['a']} + {payload['b']}",
    "engine": "local-python"
}
json.dump(result, sys.stdout)
```

---

## Gateway Binding

Routes invocations internally through the gateway's own routing layer, allowing services to
delegate to other services on the same gateway instance without external network round-trips.

> **Note:** The gateway binding is currently handled transparently via the HTTP binding with a
> relative endpoint path resolved against `runtime.http_base_url`. A dedicated `type="gateway"`
> shorthand is planned for a future phase.

**Current pattern (internal chaining via HTTP binding):**

```xml
<binding type="http"
         method="POST"
         endpoint="/services/math.open_local_add.w4s"
         contentType="application/json"/>
```

With `http_base_url = "http://127.0.0.1:8090"`, this resolves to
`http://127.0.0.1:8090/services/math.open_local_add.w4s`.

---

## Multiple Bindings

A `.w4s` file may declare multiple bindings. The runtime tries them in declaration order and uses
the first one that succeeds:

```xml
<w4s xmlns="urn:w4ml:0.1" service="my.service">
  <bindings>
    <binding type="http" method="POST" endpoint="https://primary.example.com/tool"/>
    <binding type="local" exec="bin:python3 services/fallback.py"/>
  </bindings>
</w4s>
```

---

## Effects and Binding Choice

The declared `<effects level="..."/>` on a service is informational for agents and enforced as
an invocation gate by the gateway, but does not dictate which binding type to use. Choose the
binding type based on where the computation runs; choose the effects level based on what the
computation does:

| Effects Level | Meaning |
|---|---|
| `none` | Pure computation, no external state change |
| `read` | Reads external state (databases, APIs) but does not modify it |
| `write` | Modifies external state |
| `admin` | Privileged or irreversible operations |
