# Configuring the Gateway

This guide explains every configuration option in detail and covers common deployment patterns.

---

## Configuration File

The gateway reads a single JSON config file. Pass its path as the first argument:

```bash
cargo run -p web4-gateway -- /path/to/config.json
# or with the compiled binary:
./web4-gateway /path/to/config.json
```

If no path is given, the gateway looks for `./config.json` in the current working directory.

---

## Minimal Working Config

```json
{
  "server": { "bind_addr": "127.0.0.1:8080" },
  "document": { "root": "my-site", "entry_w4": "index.w4" },
  "runtime": { "http_base_url": "http://127.0.0.1:8080" },
  "security": { "jwt_secret": "REPLACE-ME", "admin_token": "REPLACE-ME" },
  "rendering": {
    "template_loader": { "allow_remote": false, "allowed_remote_hosts": [],
                         "timeout_ms": 3000, "max_bytes": 262144 }
  }
}
```

---

## `server.bind_addr`

Controls what address and port the gateway listens on.

| Value | Use case |
|---|---|
| `"127.0.0.1:8080"` | Local development — only accessible from the same machine |
| `"0.0.0.0:8080"` | All network interfaces — use behind a reverse proxy (nginx, Caddy) |

---

## `document`

| Field | Description |
|---|---|
| `root` | Path to the directory containing the W4 document files. Resolved relative to the working directory of the gateway process. |
| `entry_w4` | Filename of the entry `.w4` document within `root`. |

The gateway restricts all file access to within `root`. Requests for paths outside `root`
are rejected with `403 FORBIDDEN` (path traversal protection).

---

## `runtime.http_base_url`

Required when any service uses an HTTP binding with a relative endpoint path. The gateway
prepends this URL to relative paths.

```json
"runtime": { "http_base_url": "http://127.0.0.1:8090" }
```

Set this to the gateway's own public URL to enable internal service chaining (where one
service's HTTP binding calls another service on the same gateway).

If no service uses a relative HTTP endpoint, this field can be omitted or set to `null`.

---

## `security`

### `jwt_secret`

The HMAC-SHA256 key used to sign and verify capability tokens. Requirements:

- **Minimum:** 32 bytes of cryptographically random data.
- Do not use dictionary words, project names, or short strings.
- Generate with: `openssl rand -hex 32`

### `admin_token`

Bearer token required to call:
- `POST /consent/issue`
- `POST /consent/challenge/{id}/approve`
- `POST /consent/challenge/{id}/deny`

Generate with: `openssl rand -hex 24`

Never commit real secrets to version control. Use environment variables or a secrets manager:

```bash
# Example: read from env, write to temp config
JWT_SECRET=$(openssl rand -hex 32)
ADMIN_TOKEN=$(openssl rand -hex 24)
jq --arg j "$JWT_SECRET" --arg a "$ADMIN_TOKEN" \
   '.security.jwt_secret = $j | .security.admin_token = $a' \
   config.template.json > config.json
```

---

## `debug.enable_error_route`

```json
"debug": { "enable_error_route": true }
```

When `true`, registers `POST /errors/{code}` which returns a sample error response for the
given code. Useful for testing client error-handling logic.

**Never enable in production.** The default is `false`.

---

## `rendering.template_loader`

| Field | Default | Description |
|---|---|---|
| `allow_remote` | `false` | Enable loading templates from external URLs declared in `<link rel="template">` |
| `allowed_remote_hosts` | `[]` | When `allow_remote` is `true`, only these hostnames are permitted |
| `timeout_ms` | `3000` | Fetch timeout for remote templates |
| `max_bytes` | `262144` | Maximum template size (256 KiB) |

**Remote templates:** keep disabled unless documents explicitly link to templates on trusted,
controlled external hosts. If you enable it:

```json
"template_loader": {
  "allow_remote": true,
  "allowed_remote_hosts": ["assets.my-company.com"],
  "timeout_ms": 5000,
  "max_bytes": 524288
}
```

---

## Common Deployment Patterns

### Development

```json
{
  "server": { "bind_addr": "127.0.0.1:8090" },
  "debug": { "enable_error_route": true }
}
```

### Behind a Reverse Proxy (nginx/Caddy)

```json
{
  "server": { "bind_addr": "127.0.0.1:8090" },
  "runtime": { "http_base_url": "https://api.example.com" }
}
```

Let the reverse proxy handle TLS termination and expose port 443 externally.

### Multiple Documents

The gateway serves exactly one W4 document per process. To serve multiple documents,
run multiple gateway instances on different ports and route between them with a reverse proxy.
