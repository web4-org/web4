# Quickstart

This guide walks you through running the showcase example and exercising all three consent modes
in under five minutes.

## 1. Start the Gateway

From the repository root:

```bash
cargo run -p web4-gateway -- config.showcase.json
```

The gateway binds to `http://127.0.0.1:8090`.

---

## 2. View the Document

### HTML (browser / human)

Open in a browser:

```
http://127.0.0.1:8090/
```

You will see the **Web4 Galaxy Console** — a neon-styled demo page listing three services and
three content sections.

### W4+JSON (agent view)

```bash
curl -s -H "Accept: application/w4+json" http://127.0.0.1:8090/ | jq .
```

This returns the full document as structured JSON, including compiled service tool definitions
suitable for consumption by an AI agent.

### Raw W4ML

```bash
curl -s -H "Accept: application/w4ml+xml" http://127.0.0.1:8090/
```

Returns the raw `.w4` document.

---

## 3. Fragment Selection

Retrieve only the `math.interactive_remix` service and its enclosing context:

```bash
curl -s "http://127.0.0.1:8090/?w4fragment=service:math.interactive_remix" \
     -H "Accept: application/w4+json" | jq .
```

---

## 4. Try the Three Consent Modes

### 4a. Open — no authorization needed

```bash
curl -s -X POST \
  http://127.0.0.1:8090/services/math.open_local_add.w4s \
  -H "Content-Type: application/json" \
  -d '{"a": 3, "b": 4}' | jq .
```

Expected:

```json
{
  "sum": 7,
  "narrative": "...",
  "engine": "open"
}
```

### 4b. Capability — issue a token first

**Step 1 — issue a capability token:**

```bash
TOKEN=$(curl -s -X POST http://127.0.0.1:8090/consent/issue \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer change-this-admin-token" \
  -d '{"service_id": "math.cap_http_bridge"}' | jq -r .token)
echo "token: $TOKEN"
```

**Step 2 — invoke with the token:**

```bash
curl -s -X POST \
  http://127.0.0.1:8090/services/math.cap_http_bridge.w4s \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-W4-Agent-Id: trusted-agent" \
  -d '{"a": 10, "b": 5}' | jq .
```

### 4c. Interactive — create and approve a challenge

**Step 1 — create a challenge:**

```bash
CHALLENGE_ID=$(curl -s -X POST http://127.0.0.1:8090/consent/challenge \
  -H "Content-Type: application/json" \
  -d '{"service_id": "math.interactive_remix"}' | jq -r .challenge_id)
echo "challenge: $CHALLENGE_ID"
```

**Step 2 — approve it (simulating human approval):**

```bash
curl -s -X POST \
  http://127.0.0.1:8090/consent/challenge/$CHALLENGE_ID/approve \
  -H "Authorization: Bearer change-this-admin-token" | jq .
```

**Step 3 — invoke with the challenge ID:**

```bash
curl -s -X POST \
  http://127.0.0.1:8090/services/math.interactive_remix.w4s \
  -H "Content-Type: application/json" \
  -H "X-W4-Challenge-Id: $CHALLENGE_ID" \
  -d '{"a": 7, "b": 8}' | jq .
```

---

## 5. Health Check

```bash
curl http://127.0.0.1:8090/healthz
# {"status":"ok"}
```

---

## Next Steps

- Understand the full data model → [Web4 Model](../concepts/web4-model.md)
- Write your own W4ML document → [Authoring W4ML](../guides/authoring-w4ml.md)
- Configure the gateway for production → [Securing Services](../guides/securing-services.md)
