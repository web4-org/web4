# Policy Enforcement

Each service in a W4 page may carry a `<policy>` block that controls access, rate, and
operational constraints. The gateway enforces these policies before every invocation.

---

## Policy Elements

```xml
<policy>
  <rateLimit value="10/s"/>
  <allowOrigins value="https://example.com"/>
  <allowAgents value="trusted-agent"/>
</policy>
```

---

## `<rateLimit>`

Limits invocation frequency per service.

```xml
<rateLimit value="3/s"/>
```

**Format:** `N/unit` where `unit` is `s` (seconds), `m` (minutes), or `h` (hours).

**Enforcement:** The gateway tracks a per-service sliding window counter. When the limit is
exceeded the gateway returns:

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "rate limit exceeded",
    "retryable": true
  }
}
```

HTTP status: `429 Too Many Requests`.

**No `<rateLimit>` declared:** the service is unlimited.

---

## `<allowOrigins>`

Controls which HTTP `Origin` values may invoke the service. Used for CORS enforcement and
cross-origin agent access control.

```xml
<!-- Allow any origin -->
<allowOrigins value="*"/>

<!-- Allow a specific origin -->
<allowOrigins value="https://dashboard.example.com"/>

<!-- Allow multiple origins (space-separated) -->
<allowOrigins value="https://app.example.com https://beta.example.com"/>
```

**Enforcement:** The gateway checks the `Origin` header of the incoming request. If the header
is present and does not match any allowed origin, the gateway returns `403 FORBIDDEN`.

If `value="*"`, any origin (or no origin) is allowed.

**CORS pre-flight:** The gateway sets the `Access-Control-Allow-Origin` response header based on
the resolved allow list.

---

## `<allowAgents>`

Restricts which agent identities may invoke the service. Agents declare their identity with the
`X-W4-Agent-Id` request header.

```xml
<allowAgents value="trusted-agent"/>
```

Multiple agent IDs can be space-separated:

```xml
<allowAgents value="agent-alpha agent-beta"/>
```

**Enforcement:** The gateway reads `X-W4-Agent-Id` from the request. If the policy is set and
the header is missing or does not match any allowed ID, the gateway returns `403 FORBIDDEN`.

**No `<allowAgents>` declared:** any agent (or anonymous caller) is allowed.

---

## Effects Level

Though declared on the service rather than in `<policy>`, the `<effects>` element acts as a
first-line enforcement gate:

```xml
<effects level="write"/>
```

| Level | Meaning | Gateway behaviour |
|---|---|---|
| `none` | Pure computation | Always allowed |
| `read` | Reads external state | Allowed unless blocked by policy |
| `write` | Modifies external state | Requires capability or interactive consent |
| `admin` | Privileged / irreversible | Strictest enforcement |

If an agent runtime declares it refuses to execute services above a given effects level, the
gateway returns `403 EFFECTS_BLOCKED`.

---

## Evaluation Order

The gateway evaluates policies in this order on each invocation request:

```
1. Effects level check
2. Consent gate (open / capability / interactive)
3. allowAgents check
4. allowOrigins check
5. rateLimit check
6. Input schema validation
7. Binding execution
8. Output schema validation
```

The first failure short-circuits the pipeline and returns the appropriate error response.

---

## Policy Override Stripping

To prevent injection attacks, the gateway strips any reserved policy-override fields from the
client-provided JSON input before passing it to the binding executor. Clients cannot bypass
policy declarations by embedding override keys in the request body.

---

## Error Reference

| Policy violation | Error code | HTTP status |
|---|---|---|
| Rate limit exceeded | `RATE_LIMITED` | 429 |
| Origin not allowed | `FORBIDDEN` | 403 |
| Agent not in allowlist | `FORBIDDEN` | 403 |
| Effects level blocked | `EFFECTS_BLOCKED` | 403 |
| Consent not satisfied | `CONSENT_REQUIRED` | 403 |
