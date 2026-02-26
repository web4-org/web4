# Authoring W4ML

This guide walks through writing a complete W4 page from scratch: the main `.w4` document
and the service binding `.w4s` files.

---

## Step 1 — Create the Document Root

Organize your files:

```
my-site/
├── index.w4              # Main W4ML document
└── services/
    ├── greet.w4s         # Service binding file
    └── greet.py          # Local Python handler
```

---

## Step 2 — Write the `.w4` Document

```xml
<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="https://example.com/index.w4">

  <head>
    <title lang="en">Hello W4</title>
    <meta name="description" content="A minimal W4 page with one service."/>
    <link rel="canonical" href="https://example.com/index.w4"/>
  </head>

  <body>

    <!-- Human-readable content section -->
    <section id="intro" name="Introduction" load="eager">
      This page exposes a greeting service. Ask it for a friendly hello.
    </section>

    <!-- Shared type definitions -->
    <schema>
      <type id="GreetInput" kind="object">
        <property name="name" type="string" required="true" desc="Name to greet"/>
        <additionalProperties value="false"/>
      </type>

      <type id="GreetOutput" kind="object">
        <property name="message" type="string" required="true"/>
        <additionalProperties value="false"/>
      </type>
    </schema>

    <!-- Service declaration -->
    <service id="greet" name="Greet" kind="tool" load="eager"
             sourceRef="services/greet.w4s">
      <intent lang="en">Returns a friendly greeting for the given name.</intent>
      <input typeRef="GreetInput"/>
      <output typeRef="GreetOutput"/>
      <effects level="none"/>
      <consent mode="open"/>
      <policy>
        <allowOrigins value="*"/>
      </policy>
    </service>

  </body>
</w4>
```

### Key Rules

- The `xmlns` and `version` attributes on `<w4>` are required; omitting them is a fatal parse error.
- Every `<service>` must reference a `sourceRef` pointing to a `.w4s` file.
- Every `typeRef` must resolve to a `<type>` declared in the same document's `<schema>`.
- Section and service `id` values must be unique within the document.
- Use BCP 47 language tags on `lang` attributes (`en`, `zh-CN`, `fr`, etc.).

---

## Step 3 — Write the `.w4s` Binding File

```xml
<w4s xmlns="urn:w4ml:0.1" service="greet">
  <bindings>
    <binding type="local" exec="bin:python3 services/greet.py"/>
  </bindings>
</w4s>
```

The `service` attribute must match the `id` of the service in the parent `.w4` file.

---

## Step 4 — Write the Handler Script

```python
# services/greet.py
import json
import sys

payload = json.load(sys.stdin)
name = payload["name"]

result = {"message": f"Hello, {name}! Welcome to Web4."}
json.dump(result, sys.stdout)
```

The script reads JSON from `stdin` and writes JSON to `stdout`. The gateway validates
both against the declared schemas.

---

## Step 5 — Configure the Gateway

```json
{
  "server": { "bind_addr": "127.0.0.1:8080" },
  "document": {
    "root": "my-site",
    "entry_w4": "index.w4"
  },
  "runtime": { "http_base_url": "http://127.0.0.1:8080" },
  "security": {
    "jwt_secret": "a-long-random-secret",
    "admin_token": "another-secret"
  },
  "rendering": {
    "template_loader": { "allow_remote": false, "allowed_remote_hosts": [],
                         "timeout_ms": 3000, "max_bytes": 262144 }
  }
}
```

---

## Step 6 — Run and Test

```bash
cargo run -p web4-gateway -- config.json

# Verify the document loads
curl -H "Accept: application/w4+json" http://127.0.0.1:8080/ | jq .services

# Invoke the service
curl -X POST http://127.0.0.1:8080/services/greet.w4s \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice"}'
# → {"message": "Hello, Alice! Welcome to Web4."}
```

---

## Choosing a Consent Mode

| Scenario | Mode | Reasoning |
|---|---|---|
| Public read-only computation | `open` | No secrets, no side-effects |
| Agent-specific data access | `capability` | Token limits who can invoke |
| Irreversible or high-risk operation | `interactive` | Human must approve each invocation |

---

## Adding a Custom HTML Template

Add a `<link rel="template">` to the `<head>`:

```xml
<link rel="template" href="templates/my-theme.html.tera" type="text/x-tera"/>
```

See [Custom Templates](custom-templates.md) for how to write the template.

---

## Common Mistakes

| Mistake | Fix |
|---|---|
| `typeRef` doesn't match any `<type id>` | Check that the type ID in `typeRef` is spelled exactly as declared in `<schema>` |
| `sourceRef` path not found | Paths in `sourceRef` are relative to the document root directory |
| Service returns extra JSON fields | Add `<additionalProperties value="false"/>` or remove the extra fields from the script output |
| Parser error: missing namespace | Add `xmlns="urn:w4ml:0.1"` to the root `<w4>` element |
