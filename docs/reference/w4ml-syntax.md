# W4ML Syntax Reference

Complete element and attribute reference for W4ML v0.1. For grammar details see
Annex A of WEB4ORG-STD-0001.

---

## Document Root: `<w4>`

```xml
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="URI">
```

| Attribute | Required | Type | Description |
|---|---|---|---|
| `xmlns` | yes | URI | Must be `urn:w4ml:0.1` |
| `version` | yes | string | Must be `0.1` |
| `id` | yes | URI | Canonical URI of this document |

Extension namespaces may be declared with additional `xmlns:prefix="..."` attributes.

Children (in order): `<head>`, `<body>`. Both are required.

---

## Head: `<head>`

Container for document metadata. Must appear before `<body>`.

### `<title>`

```xml
<title lang="en">My Page Title</title>
```

| Attribute | Required | Description |
|---|---|---|
| `lang` | no | BCP 47 language tag |

Multiple `<title>` elements are allowed for multilingual documents.

### `<meta>`

```xml
<meta name="description" content="..."/>
```

| Attribute | Required | Description |
|---|---|---|
| `name` | yes | Metadata key |
| `content` | yes | Metadata value |

### `<link>`

```xml
<link rel="REL" href="URI" type="MIME"/>
```

| Attribute | Required | Description |
|---|---|---|
| `rel` | yes | Relationship type (see table below) |
| `href` | yes | Target URI |
| `type` | no | MIME type of the target |

`rel` values:

| Value | Description |
|---|---|
| `canonical` | Self-referential canonical URI |
| `import` | Import type definitions from another W4 document |
| `peer` | Declare a peer W4 page in the W4 Graph |
| `template` | Associate a Tera HTML rendering template |

---

## Body: `<body>`

Container for content and service declarations. Children may appear in any order:
`<section>`, `<schema>`, `<service>`, extension elements.

### `<section>`

```xml
<section id="ID" name="NAME" class="CLASS" load="LOAD">
  text content and W4-lite elements
</section>
```

| Attribute | Required | Type | Description |
|---|---|---|---|
| `id` | yes | string | Unique identifier within the document |
| `name` | no | string | Human display name |
| `class` | no | string | Space-separated CSS class names for rendering |
| `load` | no | enum | `eager` (default) \| `lazy` \| `never` |

Section IDs must be unique within a document. IDs may contain letters, digits, hyphens, and
underscores. Fragment addressing uses the form `section:ID`.

---

## Schema: `<schema>`

Defines shared W4Schema types. Must contain one or more `<type>` elements.

```xml
<schema>
  <type id="..." kind="...">
    ...
  </type>
</schema>
```

### `<type>`

```xml
<type id="TypeId" kind="object">
  <property name="field" type="string" required="true" desc="Field description"/>
  <additionalProperties value="false"/>
</type>
```

| Attribute | Required | Description |
|---|---|---|
| `id` | yes | Unique type identifier within the document |
| `kind` | yes | Type kind: `object` \| `array` \| `string` \| `int` \| `float` \| `bool` |

Type IDs must be unique within a document.

### `<property>` (inside `<type kind="object">`)

```xml
<property name="field" type="string" required="true" desc="Description"/>
```

| Attribute | Required | Description |
|---|---|---|
| `name` | yes | Property name |
| `type` | yes | Primitive type or `typeRef` to a declared type |
| `required` | no | `true` \| `false` (default: `false`) |
| `desc` | no | Human-readable description |

Primitive types for `type`:

| Value | JSON Schema equivalent |
|---|---|
| `string` | `{"type": "string"}` |
| `int` | `{"type": "integer"}` |
| `float` | `{"type": "number"}` |
| `bool` | `{"type": "boolean"}` |

To reference another declared type: use `typeRef="OtherTypeId"` instead of `type`.

### `<additionalProperties>` (inside `<type kind="object">`)

```xml
<additionalProperties value="false"/>
```

When `value="false"`, the compiled JSON Schema will reject any property not declared with
`<property>`.

---

## Service: `<service>`

```xml
<service id="ID" name="NAME" kind="KIND" class="CLASS" load="LOAD" sourceRef="PATH">
  <intent lang="en">...</intent>
  <input typeRef="TYPE_ID"/>
  <output typeRef="TYPE_ID"/>
  <effects level="LEVEL"/>
  <consent mode="MODE"/>
  <policy>...</policy>
  <errors>...</errors>
</service>
```

### Service Attributes

| Attribute | Required | Description |
|---|---|---|
| `id` | yes | Unique service identifier; used in fragment addressing as `service:ID` |
| `name` | no | Human display name |
| `kind` | no | Service kind; `tool` is the standard value |
| `class` | no | CSS class for rendering |
| `load` | no | `eager` (default) \| `lazy` \| `never` |
| `sourceRef` | yes | Relative path to the `.w4s` binding file |

### `<intent>`

```xml
<intent lang="en">What this service does, for agents and humans.</intent>
```

| Attribute | Required | Description |
|---|---|---|
| `lang` | no | BCP 47 language tag |

The intent text is the primary description surfaced in the compiled `application/w4+json` output
as the tool's `description`.

### `<input>` / `<output>`

```xml
<input typeRef="CalcInput"/>
<output typeRef="CalcOutput"/>
```

| Attribute | Required | Description |
|---|---|---|
| `typeRef` | yes | ID of a `<type>` declared in the document's `<schema>` |

### `<effects>`

```xml
<effects level="none"/>
```

| `level` | Meaning |
|---|---|
| `none` | Pure computation, no external state change |
| `read` | Reads external state (databases, APIs) |
| `write` | Modifies external state |
| `admin` | Privileged or irreversible operations |

### `<consent>`

```xml
<consent mode="open"/>
```

| `mode` | Description |
|---|---|
| `open` | No authorization required |
| `capability` | JWT capability token required |
| `interactive` | Human-approved challenge required |

See [Consent Modes](../concepts/consent-modes.md).

### `<policy>`

```xml
<policy>
  <rateLimit value="10/s"/>
  <allowOrigins value="https://example.com"/>
  <allowAgents value="agent-id"/>
</policy>
```

All children are optional. See [Policy Enforcement](../concepts/policy-enforcement.md).

### `<errors>`

Declares known error codes that callers should handle:

```xml
<errors>
  <error code="UNAUTHORIZED" retryable="false"/>
</errors>
```

| Attribute | Description |
|---|---|
| `code` | Standard error code string |
| `retryable` | `true` \| `false` |

---

## Service Binding File: `<w4s>`

```xml
<w4s xmlns="urn:w4ml:0.1" service="SERVICE_ID">
  <bindings>
    <binding type="TYPE" .../>
  </bindings>
</w4s>
```

### `<binding type="http">`

```xml
<binding type="http" method="POST" endpoint="URL_OR_PATH" contentType="application/json"/>
```

| Attribute | Required | Description |
|---|---|---|
| `type` | yes | `http` |
| `method` | yes | HTTP method |
| `endpoint` | yes | Full URL or relative path (resolved against `http_base_url`) |
| `contentType` | no | Request body content type (default: `application/json`) |

### `<binding type="local">`

```xml
<binding type="local" exec="bin:COMMAND [ARGS...]"/>
```

| Attribute | Required | Description |
|---|---|---|
| `type` | yes | `local` |
| `exec` | yes | `bin:COMMAND` or `mcp:SERVER_NAME` |

---

## Fragment Addressing

Fragment selectors in the `w4fragment` query parameter:

```
section:SECTION_ID
service:SERVICE_ID
```

Examples: `?w4fragment=section:intro`, `?w4fragment=service:math.add`
