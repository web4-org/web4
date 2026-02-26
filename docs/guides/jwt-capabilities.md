# JWT Capabilities

This guide covers the full lifecycle of capability tokens: issuing them, presenting them, and
understanding the JWT claims.

---

## Overview

A **capability token** is a signed JWT that authorizes its bearer to invoke a specific
`capability`-consent service. Only the gateway can issue tokens (using the admin token), and only
the gateway can verify them (using the `jwt_secret`).

```
Admin / trusted system
        │
        │  POST /consent/issue  (with admin_token)
        ▼
    Gateway ──→ signed JWT
        │
        │  (deliver token to agent out-of-band)
        ▼
    Agent
        │
        │  POST /services/my.service.w4s
        │  Authorization: Bearer <jwt>
        ▼
    Gateway ──→ verify → invoke → response
```

---

## Issuing a Token

```bash
curl -X POST http://localhost:8090/consent/issue \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_token>" \
  -d '{
    "service_id": "math.cap_http_bridge",
    "sub": "agent-42",
    "ttl_seconds": 300
  }'
```

Response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": 1700000300
}
```

### Request Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `service_id` | string | yes | Must be the `id` of a `capability`-consent service |
| `sub` | string | no | Subject claim — identifies the agent receiving the token |
| `ttl_seconds` | number | no | Token lifetime; use the smallest value that satisfies your use case |

---

## JWT Structure

The issued token is HMAC-SHA256 signed. Decoded header and payload:

```json
// Header
{
  "alg": "HS256",
  "typ": "JWT"
}

// Payload
{
  "iss": "https://example.com/page.w4",
  "sub": "agent-42",
  "exp": 1700000300,
  "scope": "invoke:math.cap_http_bridge"
}
```

| Claim | Verified by gateway | Description |
|---|---|---|
| `iss` | Yes | Must match the document's canonical URI |
| `sub` | Yes (presence) | Token subject; logged but not used for access control |
| `exp` | Yes | Unix timestamp; token is rejected after this time |
| `scope` | Yes | Must be `invoke:<service_id>` for the target service |

---

## Presenting a Token

Include the token in the `Authorization` header on every invocation:

```bash
curl -X POST http://localhost:8090/services/math.cap_http_bridge.w4s \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "X-W4-Agent-Id: agent-42" \
  -d '{"a": 10, "b": 5}'
```

If the service also has an `<allowAgents>` policy, the `X-W4-Agent-Id` header must match an
allowed agent ID.

---

## Token Rejection

The gateway rejects a token and returns `401 UNAUTHORIZED` when:

- The signature does not verify against `jwt_secret`.
- `exp` is in the past.
- `iss` does not match the document's canonical URI.
- `scope` does not contain `invoke:<target_service_id>`.

The gateway returns `403 CONSENT_REQUIRED` when:

- No `Authorization: Bearer` header is present on a `capability`-consent service.

---

## Token Lifetime Guidelines

| Scenario | Recommended TTL |
|---|---|
| One-off agent invocation | 60–300 seconds |
| Short automated pipeline | 300–900 seconds |
| Long-running agent session | Up to 3600 seconds; re-issue before expiry |
| Human delegating to an agent | Match the expected session duration |

Tokens cannot be revoked before expiry in the current implementation. Design TTLs conservatively.

---

## Scripted Token Refresh

For long-running agents that need to refresh tokens automatically:

```python
import httpx
import time

def get_token(admin_url, admin_token, service_id, ttl=300):
    r = httpx.post(
        f"{admin_url}/consent/issue",
        json={"service_id": service_id, "sub": "my-agent", "ttl_seconds": ttl},
        headers={"Authorization": f"Bearer {admin_token}"}
    )
    r.raise_for_status()
    data = r.json()
    return data["token"], data["expires_at"]

token, expires_at = get_token(ADMIN_URL, ADMIN_TOKEN, "math.cap_http_bridge")

while True:
    # Refresh 30 seconds before expiry
    if time.time() > expires_at - 30:
        token, expires_at = get_token(ADMIN_URL, ADMIN_TOKEN, "math.cap_http_bridge")

    response = httpx.post(
        INVOKE_URL,
        json=payload,
        headers={"Authorization": f"Bearer {token}", "X-W4-Agent-Id": "my-agent"}
    )
    # ... handle response
```
