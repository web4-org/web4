# Securing Services

This guide covers security best practices for deploying the web4 gateway in production or
semi-public environments.

---

## Secrets Management

### Never commit real secrets

The `jwt_secret` and `admin_token` in `config.json` must never be committed to version control.

```bash
# Generate a strong jwt_secret
openssl rand -hex 32

# Generate a strong admin_token
openssl rand -hex 24
```

Use one of these patterns to inject secrets at runtime:

**Environment variable injection:**

```bash
jq --arg j "$JWT_SECRET" --arg a "$ADMIN_TOKEN" \
   '.security.jwt_secret = $j | .security.admin_token = $a' \
   config.template.json > /run/config.json

./web4-gateway /run/config.json
```

**Docker secrets / Kubernetes secrets:** Mount the config file from a secrets volume rather than
baking it into the image.

---

## Network Exposure

### Bind to loopback in development

```json
"server": { "bind_addr": "127.0.0.1:8090" }
```

### Use a reverse proxy in production

Never expose the gateway directly on port 80/443. Instead:

1. Bind the gateway to a loopback port.
2. Use nginx, Caddy, or a cloud load balancer for TLS termination and public exposure.

```nginx
server {
    listen 443 ssl;
    server_name api.example.com;

    location / {
        proxy_pass http://127.0.0.1:8090;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## Consent Mode Selection

Choose the most restrictive consent mode that satisfies your requirements:

| Service type | Recommended mode |
|---|---|
| Public read-only computation | `open` with rate limiting |
| Agent-specific data access | `capability` |
| Any write operation | `capability` or `interactive` |
| Irreversible actions (delete, payment) | `interactive` |

Avoid using `open` for services with `effects level="write"` or higher.

---

## Rate Limiting

Add `<rateLimit>` to all publicly accessible `open`-consent services:

```xml
<policy>
  <rateLimit value="10/s"/>
  <allowOrigins value="*"/>
</policy>
```

Without rate limiting, open services can be invoked without limit by any caller.

For `capability` and `interactive` services, rate limiting is a secondary defense — the
consent mechanism is the primary control.

---

## Agent Allowlisting

For `capability`-consent services that should only be called by specific agents, add
`<allowAgents>`:

```xml
<policy>
  <allowAgents value="production-agent staging-agent"/>
</policy>
```

Agents must set `X-W4-Agent-Id` to a value on the allowlist. This is advisory trust, not
cryptographic proof — for stronger identity, combine with JWT claims verification.

---

## Local Binding Safety

Local bindings execute processes on the gateway host. Risks:

- **Input injection:** If your script constructs shell commands from input, an attacker can
  inject arbitrary commands. Never use `subprocess.call(shell=True)` with user input.
- **Resource exhaustion:** A local process can consume CPU/memory. Set resource limits with
  `ulimit` or cgroups.
- **Filesystem access:** Run scripts under a dedicated low-privilege user account.

**Minimum checklist for local binding scripts:**

- Validate input using the W4Schema (the gateway does this before calling the script, but
  defence-in-depth is valuable).
- Never construct shell command strings from input data.
- Run as a non-root user.
- Exit with a non-zero code on error; the gateway will return `INTERNAL_ERROR`.

---

## Remote Template Safety

Keep `rendering.template_loader.allow_remote` disabled unless strictly necessary:

```json
"template_loader": { "allow_remote": false }
```

If you must enable it, populate an explicit allowlist:

```json
"template_loader": {
  "allow_remote": true,
  "allowed_remote_hosts": ["assets.my-company.com"]
}
```

Remote templates that load malicious content could expose sensitive template context variables.

---

## Debug Route

Ensure the debug route is off in production:

```json
"debug": { "enable_error_route": false }
```

---

## TLS

The gateway itself does not terminate TLS. Always deploy behind a TLS-terminating reverse proxy
when handling sensitive data. Do not expose the gateway's bind port publicly over plain HTTP
in production.

---

## Security Checklist

- [ ] `jwt_secret` is at least 32 bytes of random data
- [ ] `admin_token` is strong and not shared with untrusted parties
- [ ] `config.json` is excluded from version control (add to `.gitignore`)
- [ ] Gateway binds to `127.0.0.1`, not `0.0.0.0`, if behind a reverse proxy
- [ ] TLS is terminated by a reverse proxy
- [ ] `debug.enable_error_route` is `false`
- [ ] `rendering.template_loader.allow_remote` is `false` (or has an explicit allowlist)
- [ ] All `open` services have `<rateLimit>` policy
- [ ] Local binding scripts run under a dedicated non-root user
- [ ] Local binding scripts do not use shell=True with user input
