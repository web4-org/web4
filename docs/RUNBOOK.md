# Web4 Gateway Runbook

## Binary startup config

`web4-gateway` now uses a single JSON config file.

- Default path: `./config.json`
- Optional custom path: `cargo run -p web4-gateway -- /path/to/config.json`

Config layout:

```json
{
  "server": {
    "bind_addr": "127.0.0.1:8080"
  },
  "document": {
    "root": "examples/import-peer",
    "entry_w4": "import-peer.w4"
  },
  "runtime": {
    "http_base_url": "http://127.0.0.1:8080"
  },
  "security": {
    "jwt_secret": "change-me",
    "admin_token": "change-this-admin-token"
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

Example:

```bash
cargo run -p web4-gateway
```

## Library API

`web4-core` provides a stable builder API:

- `RuntimeOptions`
- `build_default_runtime(options)`
- `DefaultServiceRuntime`

These APIs allow embedding runtime construction without depending on gateway internals.
