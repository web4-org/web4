# Error Codes

All gateway error responses follow a standard JSON envelope (Clause 16 of WEB4ORG-STD-0001):

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable description.",
    "retryable": false,
    "details": { }
  }
}
```

| Field | Type | Description |
|---|---|---|
| `code` | string | Machine-readable error code (see table below) |
| `message` | string | Human-readable description of the error |
| `retryable` | bool | `true` if the same request might succeed on retry |
| `details` | object | Optional additional context; omitted when empty |

---

## Standard Error Codes

| Code | HTTP Status | Retryable | Meaning |
|---|---|---|---|
| `INVALID_ARGUMENT` | 400 Bad Request | false | The request body or parameters failed schema validation |
| `UNAUTHORIZED` | 401 Unauthorized | false | The provided credential (JWT) is invalid, expired, or has an invalid signature |
| `FORBIDDEN` | 403 Forbidden | false | The caller is authenticated but not permitted (agent not in allowlist, origin blocked) |
| `NOT_FOUND` | 404 Not Found | false | The requested document path, service, or fragment does not exist |
| `CONSENT_REQUIRED` | 403 Forbidden | false | The service requires a capability token or approved challenge that was not provided |
| `RATE_LIMITED` | 429 Too Many Requests | true | The service's rate limit has been exceeded; retry after a short delay |
| `EFFECTS_BLOCKED` | 403 Forbidden | false | The service's declared effects level is blocked by the agent runtime or policy |
| `INTERNAL_ERROR` | 500 Internal Server Error | false | An unexpected server-side error occurred |

---

## Error Examples

### `INVALID_ARGUMENT`

```json
{
  "error": {
    "code": "INVALID_ARGUMENT",
    "message": "input validation failed: 'a' is required",
    "retryable": false
  }
}
```

Triggered when the request body does not conform to the service's declared input schema.

### `UNAUTHORIZED`

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "JWT signature verification failed",
    "retryable": false
  }
}
```

Triggered when a `capability` mode service receives a token with an invalid signature or
expired `exp` claim.

### `CONSENT_REQUIRED`

```json
{
  "error": {
    "code": "CONSENT_REQUIRED",
    "message": "capability token required for this service",
    "retryable": false
  }
}
```

Triggered when:
- A `capability` service is called without an `Authorization: Bearer` header.
- An `interactive` service is called without an `X-W4-Challenge-Id` header.
- The provided challenge has not been approved or has expired.

### `RATE_LIMITED`

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "rate limit exceeded",
    "retryable": true
  }
}
```

The only standard retryable error. Callers should implement exponential back-off.

### `NOT_FOUND`

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "fragment 'service:unknown.svc' not found",
    "retryable": false
  }
}
```

### `INTERNAL_ERROR`

```json
{
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "binding executor returned an unexpected error",
    "retryable": false
  }
}
```

---

## Extension Codes

Implementations may emit extension error codes for domain-specific failure modes. Extension
codes are arbitrary strings not in the standard table above. Their HTTP status maps to `500`
unless the implementation documents otherwise.

Services may declare expected error codes in their `<errors>` block:

```xml
<errors>
  <error code="UNAUTHORIZED" retryable="false"/>
  <error code="MY_CUSTOM_ERROR" retryable="true"/>
</errors>
```

This signals to agents which errors are expected and how to handle them.

---

## Handling Errors in Agent Code

```python
import httpx

response = httpx.post(endpoint, json=payload)
if not response.is_success:
    err = response.json()["error"]
    if err["code"] == "RATE_LIMITED":
        time.sleep(backoff)
        # retry
    elif err["code"] == "CONSENT_REQUIRED":
        # obtain token or challenge, then retry
        pass
    elif not err["retryable"]:
        raise RuntimeError(f"W4 error: {err['code']} — {err['message']}")
```
