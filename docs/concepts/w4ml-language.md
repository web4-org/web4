# W4ML Language

W4ML (Web4 Markup Language) is the XML-based document format at the heart of Web4.
Its MIME type is `application/w4ml+xml` and its file extension is `.w4` for documents
and `.w4s` for service binding files.

> W4ML is defined by its own grammar (Annex A of WEB4ORG-STD-0001) and does not require
> a conformant XML 1.0 parser, though it draws on XML conventions for readability and
> toolchain compatibility.

---

## Document File (`.w4`)

A `.w4` file is the main document. It contains the full page: head, body, schema, and service
declarations.

### Root element

```xml
<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="https://example.com/page.w4">
  ...
</w4>
```

Required attributes on `<w4>`:

| Attribute | Required | Value |
|---|---|---|
| `xmlns` | yes | `urn:w4ml:0.1` |
| `version` | yes | `0.1` |
| `id` | yes | Canonical URI of this document |

Additional namespaces may be declared for extension elements, e.g.:

```xml
<w4 xmlns="urn:w4ml:0.1" xmlns:fx="urn:web4:effects:demo" ...>
```

---

## Head Elements

All children of `<head>`:

### `<title>`

```xml
<title lang="en">My W4 Page</title>
```

`lang` follows BCP 47. Multiple `<title>` elements with different `lang` values are permitted
for multilingual documents.

### `<meta>`

```xml
<meta name="description" content="A page that exposes math tools."/>
<meta name="theme-color" content="#0b1020"/>
```

Arbitrary key-value pairs surfaced in the normalized model and in `application/w4+json`.

### `<link>`

```xml
<!-- Canonical self-reference -->
<link rel="canonical" href="https://example.com/page.w4"/>

<!-- Import schema types from another document -->
<link rel="import" href="https://example.com/common.types.w4" type="application/w4ml+xml"/>

<!-- Peer node in the W4 Graph -->
<link rel="peer" href="https://peer.example.com/index.w4" type="application/w4ml+xml"/>

<!-- HTML rendering template -->
<link rel="template" href="templates/my-theme.html.tera" type="text/x-tera"/>
```

`rel` values:

| Value | Meaning |
|---|---|
| `canonical` | Authoritative URI for this document |
| `import` | Import schema/type definitions |
| `peer` | Declare a peer W4 page |
| `template` | Associate a Tera HTML rendering template |

---

## Body Elements

### `<section>`

Content container for human-readable prose and agent knowledge.

```xml
<section id="intro" name="Introduction" class="hero" load="eager">
  This page demonstrates the three consent modes.
</section>
```

Attributes: `id` (required), `name`, `class`, `load` (`eager`|`lazy`|`never`).

Section text content follows **W4-lite**: plain text plus a defined set of inline/block
elements (see the spec, Clause 9). Unrecognized elements in extension namespaces are preserved
but not interpreted.

### `<schema>`

Defines shared W4Schema types referenced by services.

```xml
<schema>
  <type id="MyInput" kind="object">
    <property name="query" type="string" required="true" desc="Search query"/>
    <additionalProperties value="false"/>
  </type>
</schema>
```

See [W4ML Syntax Reference](../reference/w4ml-syntax.md#schema) for the full type system.

### `<service>`

Declares a callable capability. Execution details live in the referenced `.w4s` file.

```xml
<service id="search" name="Search" kind="tool" load="eager"
         sourceRef="services/search.w4s">
  <intent lang="en">Search the knowledge base and return matching results.</intent>
  <input typeRef="MyInput"/>
  <output typeRef="MyOutput"/>
  <effects level="read"/>
  <consent mode="open"/>
  <policy>
    <rateLimit value="10/s"/>
    <allowOrigins value="*"/>
  </policy>
</service>
```

---

## Service Binding File (`.w4s`)

A `.w4s` file specifies the concrete execution mapping for a single service.
It is referenced by `sourceRef` on the `<service>` element.

```xml
<w4s xmlns="urn:w4ml:0.1" service="search">
  <bindings>
    <binding type="local" exec="bin:python3 services/search.py"/>
  </bindings>
</w4s>
```

Root element attributes:

| Attribute | Required | Value |
|---|---|---|
| `xmlns` | yes | `urn:w4ml:0.1` |
| `service` | yes | Must match the `id` of the service in the `.w4` file |

See [Bindings](bindings.md) for all binding types.

---

## Extension Namespaces

Unknown elements in declared extension namespaces are preserved in the parse tree and passed
through to the normalized model. This allows vendor-specific tooling without breaking conformant
parsers:

```xml
<w4 xmlns="urn:w4ml:0.1" xmlns:acme="urn:acme:w4ext:1" ...>
  <body>
    <acme:analytics track="pageview"/>
    ...
  </body>
</w4>
```

---

## Character Encoding

All W4ML files shall be encoded in UTF-8. The XML declaration is recommended:

```xml
<?xml version="1.0" encoding="UTF-8"?>
```
