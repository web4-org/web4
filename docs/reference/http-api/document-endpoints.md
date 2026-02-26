# Document Endpoints

## `GET /`

Retrieve the root W4 document. The response format depends on the `Accept` header.

### Request

```
GET / HTTP/1.1
Accept: application/w4+json
```

**Query parameters:**

| Parameter | Type | Description |
|---|---|---|
| `w4fragment` | string | Fragment selector, e.g. `section:intro` or `service:math.add` |

### Response

**`Accept: application/w4+json`** — compiled JSON for agents:

```
HTTP/1.1 200 OK
Content-Type: application/w4+json
Vary: Accept
```

```json
{
  "id": "https://example.com/page.w4",
  "title": "My W4 Page",
  "sections": [
    {
      "id": "intro",
      "name": "Introduction",
      "load": "eager",
      "content": "..."
    }
  ],
  "services": [
    {
      "id": "math.add",
      "name": "Add Numbers",
      "description": "Adds two integers.",
      "kind": "tool",
      "input": { "type": "object", "properties": { ... } },
      "output": { "type": "object", "properties": { ... } },
      "effects": "none",
      "consent": "open",
      "x-w4": { "load": "eager", "class": "svc-open" }
    }
  ],
  "imports": [],
  "peers": []
}
```

**`Accept: application/w4ml+xml`** — raw W4ML source:

```
HTTP/1.1 200 OK
Content-Type: application/w4ml+xml
Vary: Accept
```

Body: the raw `.w4` file content.

**`Accept: text/html`** (or no `Accept`) — rendered HTML:

```
HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Vary: Accept
```

Body: HTML rendered via the document's associated Tera template (or the built-in default).

### Fragment Selection

```
GET /?w4fragment=service:math.add HTTP/1.1
Accept: application/w4+json
```

Returns only the specified fragment embedded in the same response envelope. Unknown fragments
return `404 NOT_FOUND`.

### Error Responses

| Condition | Code | HTTP |
|---|---|---|
| Fragment not found | `NOT_FOUND` | 404 |
| Invalid fragment selector syntax | `INVALID_ARGUMENT` | 400 |

---

## `GET /{source_ref}`

Retrieve a document by its source reference path. The path must exactly match the
`document.entry_w4` configured in the gateway. This endpoint is provided for cases
where clients address the document by its file path rather than the root `/`.

```
GET /showcase.w4 HTTP/1.1
Accept: application/w4+json
```

Behaviour is identical to `GET /` for matching paths. Non-matching paths return `404 NOT_FOUND`.

---

## `GET /healthz`

Health probe endpoint for load balancers and monitoring systems.

### Request

```
GET /healthz HTTP/1.1
```

### Response

```
HTTP/1.1 200 OK
Content-Type: application/json
```

```json
{"status": "ok"}
```

This endpoint requires no authentication and performs no I/O. It always returns `200 ok`
if the gateway process is running.
