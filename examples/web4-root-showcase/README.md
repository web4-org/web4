# Web4 Root Showcase

A runnable demonstration of all core Web4 capabilities in a single W4 page.

## Start the Gateway

Run from the repository root:

```bash
cargo run -p web4-gateway -- config.showcase.json
```

## View the Document

| URL | Description |
|---|---|
| http://127.0.0.1:8090/ | HTML render (neon-themed demo page) |
| `curl -H "Accept: application/w4+json" http://127.0.0.1:8090/` | W4+JSON agent view |
| http://127.0.0.1:8090/?w4fragment=service:math.interactive_remix | Fragment selection |

## Try the Three Consent Modes

### Open — no authorization required

```bash
curl -X POST http://127.0.0.1:8090/services/math.open_local_add.w4s \
  -H "Content-Type: application/json" \
  -d '{"a": 3, "b": 4}'
```

### Capability — issue a token, then invoke

```bash
# Step 1: issue token
TOKEN=$(curl -s -X POST http://127.0.0.1:8090/consent/issue \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer change-this-admin-token" \
  -d '{"service_id": "math.cap_http_bridge"}' | jq -r .token)

# Step 2: invoke with token
curl -X POST http://127.0.0.1:8090/services/math.cap_http_bridge.w4s \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-W4-Agent-Id: trusted-agent" \
  -d '{"a": 10, "b": 5}'
```

### Interactive — create challenge, approve, then invoke

```bash
# Step 1: create challenge
CHALLENGE_ID=$(curl -s -X POST http://127.0.0.1:8090/consent/challenge \
  -H "Content-Type: application/json" \
  -d '{"service_id": "math.interactive_remix"}' | jq -r .challenge_id)

# Step 2: approve (simulates human approval)
curl -s -X POST "http://127.0.0.1:8090/consent/challenge/$CHALLENGE_ID/approve" \
  -H "Authorization: Bearer change-this-admin-token"

# Step 3: invoke with approved challenge
curl -X POST http://127.0.0.1:8090/services/math.interactive_remix.w4s \
  -H "Content-Type: application/json" \
  -H "X-W4-Challenge-Id: $CHALLENGE_ID" \
  -d '{"a": 7, "b": 8}'
```

## File Structure

```
web4-root-showcase/
├── showcase.w4                          Main W4ML document
├── showcase-default-template.w4         Same document without custom template (uses built-in)
├── services/
│   ├── math.open_local_add.w4s         open consent + local binding
│   ├── math.cap_http_bridge.w4s        capability consent + HTTP binding (bridges to open service)
│   ├── math.interactive_remix.w4s      interactive consent + local binding
│   ├── ops.shadow_hidden.w4s           load=never service (statistics demo)
│   └── calc_engine.py                  Python handler for local bindings
└── templates/
    └── neon-showcase.html.tera         Custom dark/neon Tera HTML template
```

## What This Showcase Demonstrates

| Feature | Where |
|---|---|
| Fragment selection (`?w4fragment=...`) | Query the URL with `w4fragment=service:math.interactive_remix` |
| W4Schema constraints | `CalcInput` and `CalcOutput` types declared in `showcase.w4` |
| Service policy (`<rateLimit>`, `<allowOrigins>`, `<allowAgents>`) | `math.cap_http_bridge` service |
| `open` consent with local binding | `math.open_local_add` |
| `capability` consent with HTTP binding | `math.cap_http_bridge` (bridges internally to open service) |
| `interactive` consent with challenge lifecycle | `math.interactive_remix` |
| `load=never` statistics tracking | `ops.shadow_hidden` service |
| Custom Tera HTML template | `templates/neon-showcase.html.tera` |
| JWT capability token flow | `/consent/issue` → `Authorization: Bearer` |
| Interactive challenge flow | `/consent/challenge` → approve → `X-W4-Challenge-Id` |
