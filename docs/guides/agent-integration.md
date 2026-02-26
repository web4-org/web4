# Agent Integration

This guide explains how AI agents (LLM-based systems, tool-calling frameworks, custom scripts)
can discover and invoke W4 services.

---

## Discovery via `application/w4+json`

An agent discovers a W4 page's capabilities by requesting the `application/w4+json`
representation:

```python
import httpx

response = httpx.get(
    "http://localhost:8090/",
    headers={"Accept": "application/w4+json"}
)
doc = response.json()
```

The response contains a compiled list of services in a format compatible with OpenAI-like
tool-calling frameworks:

```json
{
  "id": "https://example.com/page.w4",
  "title": "My W4 Page",
  "services": [
    {
      "id": "math.add",
      "name": "Add Numbers",
      "description": "Adds two integers and returns their sum.",
      "kind": "tool",
      "input": {
        "type": "object",
        "properties": {
          "a": {"type": "integer", "description": "Left operand"},
          "b": {"type": "integer", "description": "Right operand"}
        },
        "required": ["a", "b"],
        "additionalProperties": false
      },
      "output": { ... },
      "effects": "none",
      "consent": "open",
      "x-w4": {
        "load": "eager",
        "invoke_url": "http://localhost:8090/services/math.add.w4s"
      }
    }
  ]
}
```

---

## Building a Tool List for OpenAI-Compatible APIs

The compiled service format maps directly to OpenAI function/tool format:

```python
def w4_services_to_tools(doc):
    tools = []
    for svc in doc["services"]:
        if svc.get("load") == "never":
            continue
        tools.append({
            "type": "function",
            "function": {
                "name": svc["id"].replace(".", "_"),  # dots may be invalid in some APIs
                "description": svc["description"],
                "parameters": svc["input"]
            }
        })
    return tools

tools = w4_services_to_tools(doc)
```

---

## Invoking a Service

Use the `invoke_url` from the `x-w4` metadata block, or construct it from the service ID
and gateway base URL.

### Open Consent

```python
def invoke_open(base_url, source_ref, payload):
    r = httpx.post(
        f"{base_url}/{source_ref}",
        json=payload,
        headers={"Content-Type": "application/json"}
    )
    r.raise_for_status()
    return r.json()

result = invoke_open(
    "http://localhost:8090",
    "services/math.open_local_add.w4s",
    {"a": 3, "b": 4}
)
```

### Capability Consent

```python
def invoke_with_token(base_url, source_ref, payload, token, agent_id=None):
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    if agent_id:
        headers["X-W4-Agent-Id"] = agent_id

    r = httpx.post(f"{base_url}/{source_ref}", json=payload, headers=headers)
    r.raise_for_status()
    return r.json()
```

### Interactive Consent

```python
def invoke_interactive(base_url, source_ref, payload, service_id, approve_fn):
    """
    approve_fn(challenge_id) -> None: call your approval mechanism
    """
    # 1. Create challenge
    r = httpx.post(f"{base_url}/consent/challenge",
                   json={"service_id": service_id, "ttl_seconds": 120})
    r.raise_for_status()
    challenge_id = r.json()["challenge_id"]

    # 2. Wait for human approval (out-of-band)
    approve_fn(challenge_id)

    # 3. Invoke
    r = httpx.post(
        f"{base_url}/{source_ref}",
        json=payload,
        headers={
            "Content-Type": "application/json",
            "X-W4-Challenge-Id": challenge_id
        }
    )
    r.raise_for_status()
    return r.json()
```

---

## Error Handling

All errors follow a standard envelope. Agents should inspect the `code` and `retryable` fields:

```python
def invoke_service(url, payload, headers):
    r = httpx.post(url, json=payload, headers=headers)

    if r.is_success:
        return r.json()

    err = r.json()["error"]
    code = err["code"]
    retryable = err.get("retryable", False)

    if code == "RATE_LIMITED" and retryable:
        time.sleep(2)
        return invoke_service(url, payload, headers)  # retry once
    elif code == "CONSENT_REQUIRED":
        raise ConsentRequiredError(err["message"])
    elif code == "INVALID_ARGUMENT":
        raise ValueError(f"Input validation failed: {err['message']}")
    else:
        raise RuntimeError(f"W4 error {code}: {err['message']}")
```

---

## Fragment-Based Discovery

For large documents, agents can request just the fragment they need:

```python
# Get only the service relevant to the current task
r = httpx.get(
    "http://localhost:8090/",
    headers={"Accept": "application/w4+json"},
    params={"w4fragment": "service:math.add"}
)
svc = r.json()
```

---

## Respecting Load Strategy

Agents should respect the `load` field on services and sections:

| Value | Agent behaviour |
|---|---|
| `eager` | Include in active context / tool registry on initial load |
| `lazy` | Register lazily; only activate when the tool is needed |
| `never` | Do not include in the tool registry |

---

## Effect Level Planning

Before invoking a service, agents should check the `effects` level and factor it into planning:

```python
SAFE_EFFECTS = {"none", "read"}

for svc in doc["services"]:
    if svc["effects"] not in SAFE_EFFECTS:
        print(f"Service {svc['id']} has effects={svc['effects']}, skipping autonomous invocation")
```

---

## Complete Discovery + Invocation Example

```python
import httpx

BASE = "http://localhost:8090"

# 1. Discover
doc = httpx.get(BASE, headers={"Accept": "application/w4+json"}).json()

# 2. Find an open, safe service
svc = next(
    s for s in doc["services"]
    if s["consent"] == "open" and s["effects"] == "none"
)
invoke_url = svc["x-w4"]["invoke_url"]

# 3. Invoke
result = httpx.post(invoke_url, json={"a": 5, "b": 6},
                    headers={"Content-Type": "application/json"}).json()
print(result)
```
