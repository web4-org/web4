# Service Invoke

## `POST /{source_ref}`

Invoke a service by its source reference path (the `sourceRef` attribute on the `<service>` element).

---

### Request

```
POST /services/math.open_local_add.w4s HTTP/1.1
Content-Type: application/json
```

```json
{
  "a": 3,
  "b": 4
}
```

**Consent headers** (required depending on the service's consent mode):

| Header | Required when | Description |
|---|---|---|
| `Authorization: Bearer <token>` | `consent mode="capability"` | JWT capability token issued by `/consent/issue` |
| `X-W4-Challenge-Id: <id>` | `consent mode="interactive"` | Challenge ID in `approved` state |
| `X-W4-Agent-Id: <id>` | service has `<allowAgents>` | Agent identity string for allowlist check |
| `Origin: <origin>` | service has `<allowOrigins>` | HTTP origin for CORS enforcement |

---

### Invocation Pipeline

The gateway executes the following steps in order:

1. Locate the service by resolving `source_ref` to a `.w4s` file within the document root.
2. Load the service definition from the parent `.w4` document.
3. Check declared `<effects>` level.
4. Evaluate the consent gate (`open` / `capability` / `interactive`).
5. Evaluate `<allowAgents>` policy.
6. Evaluate `<allowOrigins>` policy.
7. Evaluate `<rateLimit>` policy.
8. Validate the request body against the service's declared `<input>` schema.
9. Execute the binding (HTTP, Local, or Gateway).
10. Validate the binding output against the service's declared `<output>` schema.
11. Return the validated output.

---

### Response — Success

```
HTTP/1.1 200 OK
Content-Type: application/json
```

```json
{
  "sum": 7,
  "narrative": "Computed 3 + 4",
  "engine": "local-python"
}
```

The response body is the JSON output from the binding executor, validated against the service's
declared output schema.

---

### Response — Error

All errors follow the standard envelope:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "...",
    "retryable": false
  }
}
```

| Condition | Code | HTTP |
|---|---|---|
| Request body fails input schema | `INVALID_ARGUMENT` | 400 |
| Missing/invalid JWT (capability mode) | `UNAUTHORIZED` | 401 |
| Agent not in allowlist | `FORBIDDEN` | 403 |
| Origin not allowed | `FORBIDDEN` | 403 |
| Consent not satisfied | `CONSENT_REQUIRED` | 403 |
| Effects level blocked | `EFFECTS_BLOCKED` | 403 |
| Service source ref not found | `NOT_FOUND` | 404 |
| Rate limit exceeded | `RATE_LIMITED` | 429 |
| Binding executor error | `INTERNAL_ERROR` | 500 |

---

### Examples

**Open consent (no auth):**

```bash
curl -X POST http://localhost:8090/services/math.open_local_add.w4s \
  -H "Content-Type: application/json" \
  -d '{"a": 10, "b": 20}'
```

**Capability consent:**

```bash
curl -X POST http://localhost:8090/services/math.cap_http_bridge.w4s \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-W4-Agent-Id: my-agent" \
  -d '{"a": 10, "b": 20}'
```

**Interactive consent:**

```bash
curl -X POST http://localhost:8090/services/math.interactive_remix.w4s \
  -H "Content-Type: application/json" \
  -H "X-W4-Challenge-Id: abc123" \
  -d '{"a": 10, "b": 20}'
```

---

### Input Validation

The gateway validates the request body against the JSON Schema compiled from the service's
`<input typeRef="...">` declaration. The validation is strict:

- Missing required fields → `INVALID_ARGUMENT`
- Extra fields (when `additionalProperties: false`) → `INVALID_ARGUMENT`
- Wrong field types → `INVALID_ARGUMENT`

Validation errors include the field path and constraint violation in the `message` field.
