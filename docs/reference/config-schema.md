# Gateway Configuration Schema

The gateway is configured via a single JSON file. The default path is `./config.json`;
a custom path can be passed as a command-line argument:

```bash
cargo run -p web4-gateway -- /path/to/config.json
```

---

## Full Schema

```json
{
  "server": {
    "bind_addr": "127.0.0.1:8080"
  },
  "document": {
    "root": "examples/my-site",
    "entry_w4": "index.w4"
  },
  "runtime": {
    "http_base_url": "http://127.0.0.1:8080"
  },
  "security": {
    "jwt_secret": "CHANGE-ME-IN-PRODUCTION",
    "admin_token": "CHANGE-ME-IN-PRODUCTION"
  },
  "debug": {
    "enable_error_route": false
  },
  "rendering": {
    "template_loader": {
      "allow_remote": false,
      "allowed_remote_hosts": [],
      "timeout_ms": 3000,
      "max_bytes": 262144
    }
  }
}
```

---

## `server`

| Field | Type | Default | Description |
|---|---|---|---|
| `bind_addr` | string | — | TCP address and port to listen on, e.g. `"127.0.0.1:8080"` |

**Examples:**

```json
"bind_addr": "127.0.0.1:8080"    // loopback only (development)
"bind_addr": "0.0.0.0:8080"      // all interfaces (production behind a reverse proxy)
```

---

## `document`

| Field | Type | Default | Description |
|---|---|---|---|
| `root` | string | — | Filesystem path to the W4 document root directory |
| `entry_w4` | string | — | Filename of the entry `.w4` document within `root` |

`root` is resolved relative to the current working directory when the binary is invoked.
Path traversal outside of `root` is rejected by the gateway.

**Example:**

```json
"document": {
  "root": "examples/web4-root-showcase",
  "entry_w4": "showcase.w4"
}
```

---

## `runtime`

| Field | Type | Default | Description |
|---|---|---|---|
| `http_base_url` | string | `null` | Base URL used to resolve relative HTTP binding endpoints |

When a service uses an HTTP binding with a relative `endpoint` (e.g., `/services/other.w4s`),
the gateway prepends `http_base_url` to form the full URL. Set this to the gateway's own
public URL to enable internal service chaining.

**Example:**

```json
"runtime": {
  "http_base_url": "http://127.0.0.1:8090"
}
```

---

## `security`

| Field | Type | Default | Description |
|---|---|---|---|
| `jwt_secret` | string | — | HMAC-SHA256 secret for signing and verifying capability JWTs |
| `admin_token` | string | — | Bearer token required to call admin endpoints (`/consent/issue`, challenge approve/deny) |

**Security requirements:**

- Both values must be changed from the defaults before deploying to any non-local environment.
- `jwt_secret` should be at least 32 bytes of high-entropy random data.
- `admin_token` should be treated as a secret credential. Do not commit real values to version control.

**Never commit `config.json` with real secrets.** Use environment-variable substitution or a
secrets manager in production.

---

## `debug`

| Field | Type | Default | Description |
|---|---|---|---|
| `enable_error_route` | bool | `false` | If `true`, registers `POST /errors/{code}` for testing error response shapes |

The debug error route should never be enabled in production.

---

## `rendering`

Controls the HTML template loader.

```json
"rendering": {
  "template_loader": {
    "allow_remote": false,
    "allowed_remote_hosts": [],
    "timeout_ms": 3000,
    "max_bytes": 262144
  }
}
```

### `rendering.template_loader`

| Field | Type | Default | Description |
|---|---|---|---|
| `allow_remote` | bool | `false` | Whether to allow loading templates from remote URLs |
| `allowed_remote_hosts` | string[] | `[]` | Allowlist of remote hostnames when `allow_remote` is `true` |
| `timeout_ms` | number | `3000` | HTTP fetch timeout for remote templates (milliseconds) |
| `max_bytes` | number | `262144` | Maximum template file size in bytes (default: 256 KiB) |

**Security note:** Keep `allow_remote` disabled (`false`) unless your W4 documents explicitly
link to templates on trusted external hosts. If you enable it, populate `allowed_remote_hosts`
with an explicit allowlist rather than leaving it empty (which may permit any host depending on
the implementation version).

---

## Showcase Example Config

The file `config.showcase.json` in the repository root is a ready-to-use example:

```json
{
  "server": { "bind_addr": "127.0.0.1:8090" },
  "document": {
    "root": "examples/web4-root-showcase",
    "entry_w4": "showcase.w4"
  },
  "runtime": { "http_base_url": "http://127.0.0.1:8090" },
  "security": {
    "jwt_secret": "change-me",
    "admin_token": "change-this-admin-token"
  },
  "debug": { "enable_error_route": false },
  "rendering": {
    "template_loader": {
      "allow_remote": false,
      "allowed_remote_hosts": [],
      "timeout_ms": 3000,
      "max_bytes": 262144
    }
  }
}
```
