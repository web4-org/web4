# Consent Modes

Every service in a W4 page declares a **consent mode** that governs whether and how it may be
invoked. The gateway enforces the mode before passing execution to the binding executor.

There are three modes, declared on the `<consent>` child element of `<service>`:

```xml
<consent mode="open"/>
<consent mode="capability"/>
<consent mode="interactive"/>
```

---

## `open`

The service requires no authorization. Any caller may invoke it.

**Use when:** the service has no side effects, is safe to expose publicly, and data sensitivity
is not a concern.

**Gateway behaviour:** the request proceeds directly to binding execution after input validation.

**Example:**

```xml
<service id="math.add" ...>
  <effects level="none"/>
  <consent mode="open"/>
  <policy>
    <allowOrigins value="*"/>
  </policy>
</service>
```

**Invocation:**

```bash
curl -X POST http://localhost:8090/services/math.open_local_add.w4s \
  -H "Content-Type: application/json" \
  -d '{"a": 3, "b": 4}'
```

---

## `capability`

The caller must present a valid **capability token** (JWT) issued by the hosting party. The token
is scoped to a specific service and carries an expiry time.

**Use when:** the service has read-level effects, is rate-limited, or requires agent identity
verification.

**Gateway behaviour:**
1. Extracts the `Authorization: Bearer <token>` header.
2. Validates JWT signature against the configured `jwt_secret`.
3. Verifies `iss`, `sub`, `exp`, and `scope` claims. The `scope` must include the service ID.
4. If valid, proceeds to binding execution.
5. If invalid or missing, returns `403 CONSENT_REQUIRED`.

**Issuing a token:**

```bash
curl -X POST http://localhost:8090/consent/issue \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{"service_id": "math.cap_http_bridge", "ttl_seconds": 300}'
```

Response:
```json
{"token": "<jwt>", "expires_at": 1700000000}
```

**Invocation with token:**

```bash
curl -X POST http://localhost:8090/services/math.cap_http_bridge.w4s \
  -H "Authorization: Bearer <jwt>" \
  -H "X-W4-Agent-Id: my-agent" \
  -H "Content-Type: application/json" \
  -d '{"a": 10, "b": 5}'
```

See [JWT Capabilities Guide](../guides/jwt-capabilities.md) for a complete walkthrough.

---

## `interactive`

The caller must present a **challenge ID** for a challenge that a human (or authorized principal)
has already approved. This mode implements a human-in-the-loop approval gate.

**Use when:** the service has write-level or higher effects and requires explicit human sign-off
before each invocation.

**Gateway behaviour:**
1. Extracts the `X-W4-Challenge-Id` header.
2. Looks up the challenge record in the in-memory store.
3. Verifies the challenge is in `approved` state and has not expired.
4. Consumes the challenge (marks it used) and proceeds to binding execution.
5. If the challenge is missing, expired, or not approved, returns `403 CONSENT_REQUIRED`.

**Challenge lifecycle:**

```
POST /consent/challenge     →  challenge_id (status: pending)
POST /consent/challenge/{id}/approve  →  status: approved   (by human / admin)
POST /consent/challenge/{id}/deny     →  status: denied

POST /services/{ref}  with X-W4-Challenge-Id  →  executes (consumes challenge)
```

**Step 1 — create:**

```bash
curl -X POST http://localhost:8090/consent/challenge \
  -H "Content-Type: application/json" \
  -d '{"service_id": "math.interactive_remix"}'
# → {"challenge_id": "abc123", "expires_at": 1700000000}
```

**Step 2 — approve:**

```bash
curl -X POST http://localhost:8090/consent/challenge/abc123/approve \
  -H "Authorization: Bearer <admin_token>"
# → {"challenge_id": "abc123", "status": "approved"}
```

**Step 3 — invoke:**

```bash
curl -X POST http://localhost:8090/services/math.interactive_remix.w4s \
  -H "X-W4-Challenge-Id: abc123" \
  -H "Content-Type: application/json" \
  -d '{"a": 7, "b": 8}'
```

---

## Comparison

| | `open` | `capability` | `interactive` |
|---|---|---|---|
| Authorization required | No | JWT token | Approved challenge |
| Token issuer | — | Gateway admin | Gateway admin |
| Human in the loop | No | No | Yes |
| Typical effects level | `none` | `read` | `write` |
| Suitable for agents | Yes | Yes (with token) | With approval |
| Rate limiting | Optional | Recommended | Optional |

---

## Error Responses

| Situation | Error Code | HTTP Status |
|---|---|---|
| Token missing on capability service | `CONSENT_REQUIRED` | 403 |
| Token expired or invalid signature | `UNAUTHORIZED` | 401 |
| Challenge not found or expired | `CONSENT_REQUIRED` | 403 |
| Challenge not yet approved | `CONSENT_REQUIRED` | 403 |
| Challenge already consumed | `CONSENT_REQUIRED` | 403 |
