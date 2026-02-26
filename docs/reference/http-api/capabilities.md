# Capabilities

Capability tokens are JWTs issued by the gateway that authorize invocation of a specific
`capability`-consent service. They carry a service scope, subject, and expiry.

---

## `POST /consent/issue`

Issue a capability token for a service.

### Request

```
POST /consent/issue HTTP/1.1
Content-Type: application/json
Authorization: Bearer <admin_token>
```

```json
{
  "service_id": "math.cap_http_bridge",
  "sub": "agent-42",
  "ttl_seconds": 300
}
```

**Headers:**

| Header | Required | Description |
|---|---|---|
| `Authorization: Bearer <token>` | yes | Gateway admin token (`security.admin_token` in config) |
| `Content-Type` | yes | `application/json` |

**Body fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `service_id` | string | yes | ID of the service for which to issue a token |
| `sub` | string | no | Subject claim (agent identifier); defaults to `"agent"` if omitted |
| `ttl_seconds` | number | no | Token lifetime in seconds; defaults to a gateway-defined value if omitted |

### Response — Success

```
HTTP/1.1 200 OK
Content-Type: application/json
```

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": 1700000300
}
```

| Field | Type | Description |
|---|---|---|
| `token` | string | Signed JWT to present in `Authorization: Bearer` on service invocations |
| `expires_at` | number | Unix timestamp (seconds) when the token expires |

### JWT Claims

The issued token contains these claims:

| Claim | Value |
|---|---|
| `iss` | Gateway document URI |
| `sub` | Value of `sub` from request body |
| `exp` | `now + ttl_seconds` |
| `scope` | `invoke:<service_id>` |

The gateway verifies all four claims (`iss`, `sub`, `exp`, `scope`) plus HMAC-SHA256 signature
on every invocation of a `capability` service.

### Response — Error

| Condition | Code | HTTP |
|---|---|---|
| Missing or invalid admin token | `UNAUTHORIZED` | 401 |
| Service ID not found in document | `NOT_FOUND` | 404 |
| Service does not use capability consent | `INVALID_ARGUMENT` | 400 |

---

### Full Example

```bash
# 1. Issue token (admin operation)
RESPONSE=$(curl -s -X POST http://localhost:8090/consent/issue \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer change-this-admin-token" \
  -d '{"service_id": "math.cap_http_bridge", "ttl_seconds": 600}')

TOKEN=$(echo $RESPONSE | jq -r .token)

# 2. Use token to invoke the service
curl -X POST http://localhost:8090/services/math.cap_http_bridge.w4s \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-W4-Agent-Id: my-agent" \
  -d '{"a": 10, "b": 5}'
```

---

### Security Notes

- The admin token (`security.admin_token`) should be stored securely and never embedded in
  client-side code or committed to version control.
- Set a short `ttl_seconds` for tokens used in automated pipelines. For interactive sessions,
  issue a fresh token per session.
- Tokens are not revocable before expiry in the current implementation. Design `ttl_seconds`
  conservatively.
