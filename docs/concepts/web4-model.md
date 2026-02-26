# Web4 Model

A **W4 page** is the fundamental unit in Web4. It is a single HTTP-addressable document that
simultaneously carries human-readable content and machine-invocable services. This page explains
the structural elements that make up a W4 page and how pages relate to each other.

---

## W4 Page Anatomy

A W4 page is authored in W4ML (a `.w4` file). At the top level it has a `<head>` and a `<body>`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="https://example.com/page.w4">
  <head>
    <!-- metadata, links, template reference -->
  </head>
  <body>
    <!-- sections, schema, services -->
  </body>
</w4>
```

---

## Head

The `<head>` holds document-level metadata and relationships:

| Element | Purpose |
|---|---|
| `<title>` | Human-readable document title, with optional `lang` attribute |
| `<meta name="..." content="...">` | Arbitrary key-value metadata (description, theme-color, etc.) |
| `<link rel="canonical" href="...">` | Canonical URI of this document |
| `<link rel="import" href="..." type="...">` | Import another W4 document's schema/types |
| `<link rel="peer" href="..." type="...">` | Declare a peer W4 page in the same graph |
| `<link rel="template" href="..." type="...">` | Associate a Tera HTML rendering template |

---

## Body Elements

### Section

A `<section>` is a content container — prose, lists, or W4-lite inline content — intended for
both human readers and agent knowledge ingestion.

```xml
<section id="intro" name="Introduction" class="hero" load="eager">
  Welcome to the Web4 demo. This page exposes three callable services.
</section>
```

Key attributes:

| Attribute | Values | Meaning |
|---|---|---|
| `id` | unique string | Fragment addressing anchor |
| `name` | string | Human display name |
| `class` | string | CSS class for rendering and semantic hinting |
| `load` | `eager` \| `lazy` \| `never` | Loading strategy (see below) |

### Schema

The `<schema>` block defines shared W4Schema types used by services:

```xml
<schema>
  <type id="CalcInput" kind="object">
    <property name="a" type="int" required="true" desc="Left operand"/>
    <property name="b" type="int" required="true" desc="Right operand"/>
    <additionalProperties value="false"/>
  </type>
</schema>
```

### Service

A `<service>` declares a callable capability:

```xml
<service id="math.add" name="Add Numbers" kind="tool" load="eager"
         sourceRef="services/math.add.w4s">
  <intent lang="en">Adds two integers and returns their sum.</intent>
  <input typeRef="CalcInput"/>
  <output typeRef="CalcOutput"/>
  <effects level="none"/>
  <consent mode="open"/>
  <policy>
    <allowOrigins value="*"/>
  </policy>
</service>
```

---

## Loading Strategy

The `load` attribute controls how agents and the rendering engine treat an element:

| Value | Agent behaviour | HTML rendering |
|---|---|---|
| `eager` | Included in active context on first load | Rendered immediately |
| `lazy` | Only ID and summary retained until requested | Folded / collapsed initially |
| `never` | Counted in statistics but not surfaced | Omitted from HTML output |

---

## W4Schema Type System

W4Schema provides the type vocabulary for service inputs and outputs:

| Primitive | Notes |
|---|---|
| `string` | UTF-8 string |
| `int` | 64-bit signed integer |
| `float` | 64-bit floating point |
| `bool` | Boolean |
| `object` | Structured object with named properties |
| `array` | Homogeneous list |

Compound types are declared with `<type id="..." kind="object">` and may contain:
- `<property name="..." type="..." required="..." desc="..."/>`
- `<additionalProperties value="false"/>` to seal the schema

---

## W4 Graph

W4 pages can reference each other, forming a **W4 Graph**:

- `<link rel="import">` — pull in schema/type definitions from another W4 page
- `<link rel="peer">` — declare a peer node; both pages appear in `application/w4+json` output
  under `imports` and `peers` keys

These references are resolvable over HTTP(S), enabling distributed, linkable knowledge graphs.

---

## Normalized Internal Model

When the gateway parses a `.w4` file it produces a **normalized internal model** — a validated,
indexed data structure containing:

- `section_ids` — ordered list of section identifiers
- `service_ids` — ordered list of service identifiers
- `type_ids` — ordered list of schema type identifiers
- Resolved `typeRef` links
- Fatal parse diagnostics

This model is the basis for all downstream operations: compilation to JSON, HTML rendering,
service invocation, and fragment selection.
