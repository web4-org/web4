# WEB4ORG-STD-0001

## Web4 Core Technical Specification v0.1: Architecture, Markup, and Service Protocol

---

| Field | Value |
|---|---|
| Document Number | WEB4ORG-STD-0001 |
| Version | 0.1 |
| Status | Technical Specification |
| Date | 2026-02-24 |
| License | Apache License, Version 2.0 |

---

## Foreword

This Technical Specification has been prepared by Web4 Organization (WEB4ORG).

This document is published under the Apache License, Version 2.0. The full license text is available at https://www.apache.org/licenses/LICENSE-2.0.

The verbal forms used in this document conform to the following conventions, derived from ISO/IEC Directives, Part 2:

- **shall** / **shall not** — indicates a requirement; deviation constitutes non-conformance.
- **should** / **should not** — indicates a recommendation; alternatives may be accepted with justification.
- **may** — indicates permission; the action is optional.
- **can** / **cannot** — indicates possibility or capability; not a requirement or permission.

Notes and examples integrated in this document are informative only and do not contain requirements.

This is the first edition of WEB4ORG-STD-0001.

---

## Introduction

The contemporary Web provides rich content for human consumption but offers weak machine-readable semantics and no unified mechanism for exposing callable capabilities alongside that content. Agent systems and tool protocols, conversely, excel at structured invocation but lack the network-layer discoverability and knowledge-bearing properties of the Web.

Web4 addresses this gap by defining a unified document format — the **W4 page** — that carries both human-readable content and machine-invocable services within a single network-addressable resource. W4 pages are published over HTTP(S), are linkable and discoverable like ordinary web pages, and are simultaneously parseable by agents as structured knowledge and callable tool registries.

This Technical Specification defines the W4 Markup Language (W4ML), the service and schema models, transport conventions, discovery mechanisms, and the security model that together constitute Web4 v0.1. The specification is intentionally minimal: it establishes the core that all conformant implementations must support, while providing extension mechanisms for future profiles and vendor-specific additions.

---

## 1. Scope

This Technical Specification specifies:

a) the Web4 Markup Language (W4ML), including document structure, formal grammar, content model, and character encoding requirements;
b) the W4Schema type system for declaring structured input and output types;
c) the service model, including service declaration, service kinds, intent, input/output binding, effects levels, consent mechanisms, policy declarations, binding types, and error declarations;
d) the progressive loading model and its semantics for both human browsers and agent runtimes;
e) HTTP(S) transport conventions and content negotiation for serving W4ML documents;
f) discovery and linking mechanisms, including well-known entry paths, HTML integration, and W4ML link elements;
g) fragment addressing syntax for identifying sections and services within a W4ML document;
h) agent-to-agent communication services;
i) interoperability targets: OpenAI-like tool JSON compilation format and implementation-defined compatibility layers (e.g., MCP);
j) the security model, including trust boundaries, default invocation policy, and transport security guidance;
k) human rendering conventions, including template association and class-based styling;
l) extensibility mechanisms via XML namespaces and profiles.

This Technical Specification is applicable to:

- implementers of W4ML parsers and validators;
- implementers of Web4 server runtimes and gateways;
- implementers of Web4 agent runtimes that discover and invoke W4 services;
- authors of W4ML documents.

This Technical Specification does not specify:

- the internal format of HTML rendering templates beyond minimal contract requirements;
- the mandatory format of capability tokens beyond a recommendation to use JWT;
- implementation programming languages or frameworks;
- Profiles for IoT, streaming, or enterprise authentication (reserved for future editions).

---

## 2. Normative References

The following documents are referred to in the text in such a way that some or all of their content constitutes requirements of this Technical Specification. For dated references, only the edition cited applies. For undated references, the latest edition of the referenced document (including any amendments) applies.

- **RFC 9110:2022** — HTTP Semantics. Fielding, R. et al. IETF.
- **RFC 9112:2022** — HTTP/1.1. Fielding, R. et al. IETF.
- **RFC 3986:2005** — Uniform Resource Identifier (URI): Generic Syntax. Berners-Lee, T. et al. IETF.
- **RFC 8259:2017** — The JavaScript Object Notation (JSON) Data Interchange Format. Bray, T. IETF.
- **RFC 7519:2015** — JSON Web Token (JWT). Jones, M. et al. IETF.
- **RFC 6838:2013** — Media Type Specifications and Registration Procedures. Freed, N. et al. IETF.
- **RFC 4648:2006** — The Base16, Base32, and Base64 Data Encodings. Josefsson, S. IETF.
- **W3C XML 1.0 (Fifth Edition):2008** — Extensible Markup Language (XML) 1.0. Bray, T. et al. W3C. (Referenced for character set and encoding definitions only; W4ML parsers are not required to be conformant XML parsers.)
- **BCP 47 (RFC 5646):2009** — Tags for Identifying Languages. Phillips, A. et al. IETF.

---

## 3. Terms and Definitions

For the purposes of this Technical Specification, the following terms and definitions apply.

**3.1 W4 page**
Document instance conforming to this Technical Specification, identified by a URI, combining content sections and service declarations within a single network-addressable resource.

**3.2 Web4 Markup Language (W4ML)**
XML-like markup language defined by this Technical Specification, with file extension `.w4` and MIME type `application/w4ml+xml`.

NOTE — W4ML is defined by its own grammar (Annex A) and does not require a conformant XML 1.0 parser. The grammar draws on XML conventions for readability and toolchain compatibility.

**3.3 section**
Content container element within the body of a W4ML document, presenting information suitable for both human reading and agent knowledge ingestion.

**3.4 W4Schema**
Structured type system defined in this Technical Specification, expressed in W4ML elements, used to declare and validate input and output types of services.

**3.5 service**
Declared callable capability within a W4ML document, comprising an intent description, typed input and output, effects declaration, consent mode, bindings, and policy.

**3.6 binding**
Concrete execution mapping for a service, specifying the transport protocol, endpoint or entrypoint, and invocation parameters sufficient to invoke the service.

**3.7 effects level**
Declared classification of the external impact that may result from invoking a service, chosen from a fixed enumeration defined in Clause 12.

**3.8 consent mode**
Declared authorization mode governing whether and how a service may be invoked, chosen from a fixed enumeration defined in Clause 13.

**3.9 policy**
Set of access-control, rate, and operational declarations associated with a service, informing agent planning and gateway governance.

**3.10 capability token**
Authorization artifact issued by a hosting party and presented by an accessing party to satisfy a `capability` consent requirement.

**3.11 W4 Graph**
Network of W4 pages and services interconnected through `link`, `import`, and fragment references resolvable over HTTP(S).

**3.12 normalized internal model**
Implementation-defined normalized data structure produced during parsing and validation, used internally as the basis for compilation to target formats such as OpenAI-like tool JSON.

**3.13 hosting party**
Agent or system that publishes and governs a W4 page and its declared services.

**3.14 accessing party**
Agent or system that discovers, loads, and invokes services declared in a W4 page.

**3.15 eager loading**
Loading strategy in which a section or service is included in the agent's active context or tool registry upon initial document load.

**3.16 lazy loading**
Loading strategy in which a section or service is deferred; only its identifier and summary are retained until explicit retrieval is requested.

**3.17 fragment address**
URI fragment component that identifies a specific section or service element within a W4ML document, conforming to the syntax defined in Clause 20.

**3.18 W4-lite**
Defined subset of inline and block content elements permitted within a `section` element, specified in Clause 9.

**3.19 profile**
Standardized extension package, identified by a URN, that adds domain-specific semantics on top of the Core defined by this Technical Specification.

---

## 4. Abbreviated Terms

| Abbreviation | Expansion |
|---|---|
| BCP | Best Current Practice |
| EBNF | Extended Backus–Naur Form |
| HTTP | Hypertext Transfer Protocol |
| HTTPS | HTTP Secure (HTTP over TLS) |
| JSON | JavaScript Object Notation |
| JWT | JSON Web Token |
| MIME | Multipurpose Internet Mail Extensions |
| MCP | Model Context Protocol |
| RPC | Remote Procedure Call |
| TLS | Transport Layer Security |
| URI | Uniform Resource Identifier |
| URL | Uniform Resource Locator |
| W4ML | Web4 Markup Language |

---

## 5. Conformance

### 5.1 Conformance targets

This Technical Specification defines requirements for the following conformance targets:

a) **W4ML document** — a document instance in the W4ML format;
b) **W4ML parser** — a software component that reads and processes W4ML documents;
c) **Web4 server runtime** — a system that serves W4ML documents over HTTP(S), performs content negotiation, and executes service bindings;
d) **Web4 agent runtime** — a system that discovers W4 pages, loads their content and services, and invokes service bindings.

A conformant implementation shall satisfy all requirements (expressed with "shall") that apply to its declared conformance target. An implementation that partially satisfies requirements shall explicitly declare which requirements it does not satisfy.

### 5.2 W4ML document conformance

A conformant W4ML document:

a) shall declare the default namespace `urn:w4ml:0.1` as the value of the `xmlns` attribute on the root `w4` element;
b) shall declare `version="0.1"` on the root `w4` element;
c) shall conform to the grammar defined in Annex A;
d) shall include explicit closing tags for all structural elements: `w4`, `head`, `body`, `section`, `service`, and `schema`;
e) shall, for every `input` and `output` element whose `typeRef` attribute references a named type, ensure that the referenced type is declared within the same document's `schema` element or within an imported schema reachable via a `link rel="import"` element.

### 5.3 W4ML parser conformance

A conformant W4ML parser:

a) shall successfully parse all documents conforming to the grammar in Annex A;
b) shall apply the error-recovery rules defined in Clause 8.3 when encountering recoverable parse errors;
c) shall not reject a document solely because it contains elements or attributes in non-default namespaces;
d) shall preserve the raw text content of unknown extension elements for pass-through and debugging purposes;
e) shall report a fatal parse error and halt when a core structural element lacks its required closing tag and no recovery rule applies.

### 5.4 Web4 server runtime conformance

A conformant Web4 server runtime:

a) shall support HTTP content negotiation as specified in Clause 18;
b) shall return the W4ML source document when the highest-preference accepted media type in a request is `application/w4ml+xml`;
c) shall return a rendered HTML document when the highest-preference accepted media type in a request is `text/html`;
d) shall include the `Vary: Accept` response header on responses that depend on content negotiation;
e) shall enforce effects and consent policies as specified in Clauses 12 and 13 before executing any service binding;
f) shall reject service invocations that violate the declared policy without invoking the binding.

### 5.5 Web4 agent runtime conformance

A conformant Web4 agent runtime:

a) shall not automatically invoke a service whose effects level is `write`, `control`, `financial`, or `unknown` without first satisfying the declared consent requirement;
b) shall treat the text content of `section` elements as informational input; section content shall not, by itself, override consent, policy, or effects enforcement;
c) shall implement progressive loading semantics as specified in Clause 17;
d) shall, when invoking a service with `consent mode="capability"`, obtain a capability token via the declared `issue` endpoint before invoking the service binding.

---

## 6. Architecture Overview

### 6.1 Design principles

Web4 is founded on the following design principles:

a) **Unified carrier** — a single W4 page carries both human-readable knowledge and machine-invocable services.
b) **Network-first** — W4 pages are linkable, discoverable, and referenceable by default over HTTP(S).
c) **Human–agent parity** — the same W4ML document is the authoritative source for both the HTML view served to humans and the structured context and tool list loaded by agents.
d) **Progressive loading** — content and services are loaded on demand, following the same principle as incremental web page rendering.
e) **Compilable interoperability** — W4ML is a source representation that can be compiled to existing tool formats (OpenAI-like JSON, MCP) without loss of Web4 semantics.
f) **Open by default, controlled by effects** — content is openly readable; service invocation is gated by the declared effects level and consent mode.

### 6.2 Architectural layers

The Web4 architecture comprises five layers:

| Layer | Components |
|---|---|
| Document | W4ML document, W4Schema, Sections, Services |
| Transport | HTTP(S), Content negotiation, TLS |
| Execution | Bindings (HTTP, Local), Consent token flow |
| Discovery | Well-known paths, Link graph, Fragment addressing |
| Compilation | OpenAI-like tool JSON, MCP compatibility |

### 6.3 Dual rendering model

A W4ML document is served from a single URI. The response depends on the `Accept` header of the request:

- An agent requesting `application/w4ml+xml` receives the raw W4ML document for parsing.
- A human browser requesting `text/html` receives a rendered HTML page.
- A client requesting `application/w4+json` may receive the compiled tool JSON view (see Clause 22).

The server determines which response to send by performing content negotiation per RFC 9110, Section 12.

---

## 7. W4ML Document Format

### 7.1 File identification

The file extension for W4ML documents is **`.w4`**.

The registered MIME type for W4ML documents is **`application/w4ml+xml`**.

### 7.2 Character encoding

W4ML documents shall be encoded in UTF-8. An optional XML declaration may appear at the beginning of the document to declare the encoding explicitly. When present, the encoding declaration shall declare `UTF-8`.

EXAMPLE
```xml
<?xml version="1.0" encoding="UTF-8"?>
```

### 7.3 Namespace declaration

The root `w4` element shall carry the attribute `xmlns="urn:w4ml:0.1"`. This namespace identifier is a URN used to identify the document as a W4ML v0.1 document; it is not required to be network-resolvable.

Extension namespaces shall be declared using the form `xmlns:prefix="urn:..."` on any element where they are first used.

### 7.4 Version attribute

The root `w4` element shall carry the attribute `version="0.1"`. Parsers encountering a version value they do not recognize should report a warning and may decline to process the document.

### 7.5 Document identifier

The root `w4` element should carry an `id` attribute whose value is the canonical HTTPS URL of the document. This value serves as the globally unique identifier of the W4 page within the W4 Graph.

EXAMPLE
```xml
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="https://example.com/index.w4">
```

### 7.6 Top-level structure

A W4ML document shall conform to the following top-level structure:

```xml
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="URI">
  <head>
    <!-- metadata elements -->
  </head>
  <body>
    <!-- section, schema, service elements -->
  </body>
</w4>
```

The `head` element shall appear before the `body` element. Both elements are required.

---

## 8. W4ML Grammar and Parsing

### 8.1 Grammar formalism

The formal grammar for W4ML is defined in Annex A using Extended Backus–Naur Form (EBNF). The grammar defines W4ML as an independent language; a conformant W4ML parser is not required to be a conformant XML 1.0 parser.

### 8.2 Attribute quoting

Attribute values may be delimited by either double-quote characters (`"`) or single-quote characters (`'`). A parser shall accept both forms. The delimiter character shall not appear unescaped within the attribute value it delimits.

Within an attribute value, the following escape sequences shall be recognized:

| Escape | Character |
|---|---|
| `&amp;` | `&` |
| `&lt;` | `<` |
| `&gt;` | `>` |
| `&quot;` | `"` |
| `&apos;` | `'` |
| `&#xNNNN;` | Unicode code point U+NNNN (hexadecimal) |

### 8.3 Error recovery

W4ML parsers shall implement the following error-recovery behaviours:

a) **Self-closing tags** — the form `<element />` shall be treated as equivalent to `<element></element>` for all elements. This applies to both void elements (e.g., `<br />`) and optionally for elements that contain no content.

b) **Unknown extension elements** — elements whose tag name is prefixed with a declared non-default namespace prefix shall not cause a parse error. The parser shall record the element's tag name, attributes, and raw text content, then continue parsing.

c) **Unknown core attributes** — unrecognized attributes on known core elements shall be ignored by the parser without causing a parse error.

d) **Whitespace** — insignificant whitespace between block-level elements within `head`, `body`, `section`, `schema`, and `service` shall be ignored.

A conformant parser shall report a **fatal parse error** and halt processing — without applying recovery — in the following cases:

- The root element is not `w4`, or the `xmlns` attribute is absent.
- A core structural element (`w4`, `head`, `body`, `section`, `service`, `schema`) lacks its closing tag with no applicable recovery rule.
- An attribute value contains a malformed escape sequence.

### 8.4 Comments

W4ML parsers shall recognize and silently discard XML-style comments of the form `<!-- ... -->`. Comments shall not appear within attribute values or element tag tokens.

---

## 9. Content Model: Section and W4-Lite

### 9.1 Section element

The `section` element is a block-level content container placed within `body`. It carries human-readable and agent-readable content.

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `id` | Yes | Unique identifier within the document. Shall match `[a-z][a-z0-9_-]*`. |
| `name` | No | Human-readable display name. |
| `class` | No | Space-separated list of class tokens for template-based styling. |
| `load` | No | Loading strategy: `eager`, `lazy`, or `never`. Default is `eager`. |

A `section` element shall have a unique `id` within the document. The same `id` value shall not be used for both a `section` and a `service`.

### 9.2 W4-lite content elements

Within a `section` element, the following W4-lite block and inline elements are permitted:

**Block elements:**

| Element | Description |
|---|---|
| `p` | Paragraph of inline content. |
| `ul` | Unordered list; contains `li` elements. |
| `ol` | Ordered list; contains `li` elements. |
| `pre` | Preformatted text block; content is treated as literal text. |

**Inline elements:**

| Element | Description |
|---|---|
| `code` | Inline code span. |
| `a` | Hyperlink; requires `href` attribute containing a URI per RFC 3986. |
| `strong` | Strong emphasis. |
| `em` | Regular emphasis. |
| `br` | Line break; void element; shall use self-closing form `<br />`. |

Content not wrapped in any W4-lite element is treated as plain text and shall be preserved by the parser.

### 9.3 Section nesting

`section` elements shall not be nested. A `section` element shall appear only as a direct child of `body`.

---

## 10. Schema System (W4Schema)

### 10.1 Schema element

The `schema` element may appear as a direct child of `body`. When present, it contains one or more `type` definitions. A document shall contain at most one `schema` element. Multiple `schema` elements in the same document shall constitute a fatal parse error.

### 10.2 Type definition

Each type within `schema` is defined by a `type` element.

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `id` | Yes | Identifier of the type. Shall match `[A-Za-z][A-Za-z0-9_]*`. Shall be unique within the document. |
| `kind` | Yes | One of the base kinds defined in Clause 10.3. |

### 10.3 Base type kinds

The following base type kinds are defined for v0.1:

| Kind | Description | JSON Schema equivalent |
|---|---|---|
| `string` | Unicode string | `{"type": "string"}` |
| `int` | Signed integer | `{"type": "integer"}` |
| `float` | IEEE 754 double-precision floating-point number | `{"type": "number"}` |
| `bool` | Boolean value | `{"type": "boolean"}` |
| `datetime` | ISO 8601 date-time string | `{"type": "string", "format": "date-time"}` |
| `uri` | URI string per RFC 3986 | `{"type": "string", "format": "uri"}` |
| `bytes` | Base64-encoded binary data per RFC 4648 | `{"type": "string", "contentEncoding": "base64"}` |
| `object` | Structured object with named properties | `{"type": "object"}` |
| `array` | Ordered list of items | `{"type": "array"}` |

For `kind="object"`, the type content shall contain `property` and optionally `additionalProperties` elements.

For `kind="array"`, a `property` element named `items` should be used to declare the element type.

### 10.4 Property declaration

Properties of an `object` type are declared using `property` elements.

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `name` | Yes | Property name. Shall match `[a-z_][a-z0-9_]*` (snake_case). |
| `type` | Yes | Base type kind (from Clause 10.3) or the `id` of a type defined in the same `schema`. |
| `required` | No | `true` or `false`. Default is `false`. |
| `desc` | No | Human-readable description of the property. |
| `default` | No | Default value, serialized as a string. |
| `min` | No | Minimum numeric value (inclusive). Applicable to `int` and `float`. |
| `max` | No | Maximum numeric value (inclusive). Applicable to `int` and `float`. |
| `minLength` | No | Minimum string length in Unicode code points. Applicable to `string`. |
| `maxLength` | No | Maximum string length in Unicode code points. Applicable to `string`. |
| `format` | No | Semantic format hint (e.g., `email`, `uuid`, `date`). Applicable to `string`. |
| `enum` | No | Comma-separated list of allowed values. |

### 10.5 Additional properties constraint

The `additionalProperties` element may appear within an `object` type definition.

| Attribute | Required | Description |
|---|---|---|
| `value` | Yes | `true` to allow additional properties; `false` to prohibit them. |

When `additionalProperties value="false"` is declared, a conformant validator shall reject input objects containing properties not explicitly declared in the type definition.

### 10.6 W4Schema to JSON Schema mapping

A conformant compiler generating JSON Schema from W4Schema shall apply the following mapping rules:

a) Each `type` element with `kind="object"` maps to a JSON Schema object with `"type": "object"`.
b) Each `property` element maps to a JSON Schema property within `"properties"`.
c) Properties with `required="true"` shall be listed in the JSON Schema `"required"` array.
d) The `min` attribute maps to `"minimum"`; `max` maps to `"maximum"`.
e) The `minLength` attribute maps to `"minLength"`; `maxLength` maps to `"maxLength"`.
f) The `format` attribute maps to `"format"`.
g) The `enum` attribute maps to `"enum"`, with values parsed by splitting on commas.
h) The `default` attribute maps to `"default"`, with the value parsed according to the property type.
i) `additionalProperties value="false"` maps to `"additionalProperties": false`.

---

## 11. Service Model

### 11.1 Service element

The `service` element declares a callable capability. It shall appear as a direct child of `body`.

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `id` | Yes | Unique service identifier. Shall match `[a-z][a-z0-9._-]*` (dot-separated). |
| `name` | No | Human-readable name. |
| `kind` | No | Service kind: `tool` (default) or `agent`. |
| `load` | No | Loading strategy: `eager`, `lazy`, or `never`. Default is `eager`. |
| `class` | No | Space-separated class tokens for template-based rendering. |
| `sourceRef` | Yes | Relative path to a `.w4s` resource under the gateway webroot. This path is the public invocation endpoint for the service. |

### 11.2 Service kinds

Two service kinds are defined for v0.1:

- **`tool`** — a general-purpose callable tool that accepts structured input and returns structured output.
- **`agent`** — an agent-to-agent communication endpoint (see Clause 21).

Future kinds (e.g., `workflow`, `stream`, `device`) shall be introduced via Profiles and shall not be defined in the Core.

### 11.3 Intent declaration

The `intent` element within a `service` provides a human- and agent-readable description of what the service does. It should be present for every service.

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `lang` | No | BCP 47 language tag (e.g., `en`, `zh`). Default is unspecified. |

Multiple `intent` elements with different `lang` values may appear within a single service to provide multilingual descriptions. A conformant agent runtime should select the intent whose `lang` best matches the agent's operating language.

### 11.4 Input and output declarations

The `input` and `output` elements declare the typed interface of a service.

| Attribute | Required | Description |
|---|---|---|
| `schema` | Yes | The `id` of a `type` defined in the local `schema` element or in an imported schema resolved via `link rel="import"`. |

Each service should declare exactly one `input` element and exactly one `output` element. A service with no `input` element is treated as accepting no structured input. A service with no `output` element is treated as returning no structured output.

### 11.5 Examples

The `examples` element may appear within a service to provide sample invocations.

Each `example` element shall contain one `call` element and one `result` element. Both shall contain content conforming to RFC 8259 JSON.

---

## 12. Effects

### 12.1 Effects declaration

The `effects` element within a `service` declares the level of external impact that may result from invoking the service.

**Syntax:**

```xml
<effects level="LEVEL"/>
```

When the `effects` element is absent from a service declaration, the effects level shall be treated as `unknown`.

### 12.2 Effects level enumeration

The following effects levels are defined for v0.1:

| Level | Description |
|---|---|
| `none` | Pure computation; no external I/O and no observable side effects outside the invocation itself. |
| `read` | Reads from an external system or data source; produces no persistent changes. |
| `write` | Writes to an external system, producing persistent changes. |
| `control` | Controls a system, device, or process; may affect availability or the physical world. |
| `financial` | Initiates a financial transaction, payment, or transfer of funds. |
| `unknown` | Effects are not declared or cannot be determined. |

### 12.3 Enforcement semantics

A conformant Web4 agent runtime shall enforce the following default policy based on the declared effects level:

a) Services with effects level `none` or `read` and consent mode `open` may be invoked automatically.
b) Services with effects level `write`, `control`, `financial`, or `unknown` shall not be invoked automatically; the agent runtime shall require explicit consent before invocation.
c) Absent a consent mechanism, a service with effects level `write`, `control`, `financial`, or `unknown` shall not be invoked.

A Web4 agent runtime may apply a more restrictive policy than specified above. It shall not apply a less restrictive policy.

---

## 13. Consent Mechanism

### 13.1 Consent declaration

The `consent` element within a `service` declares the authorization mode for invoking the service.

**Syntax:**

```xml
<consent mode="MODE">
  <!-- optional sub-elements depending on mode -->
</consent>
```

When the `consent` element is absent, the consent mode shall be treated as consistent with the effects level: services with `effects level="none"` default to `open`; all other effects levels default to `capability`.

### 13.2 Consent mode enumeration

The following consent modes are defined for v0.1:

| Mode | Description |
|---|---|
| `open` | No additional authorization is required. Should only be used with `effects level="none"` or `level="read"`. |
| `capability` | A capability token is required. Both parties express consent through token issuance and token use. |
| `interactive` | Human-in-the-loop confirmation via a UI interaction is required before invocation. |
| `deny` | Invocation is not permitted. The service declaration serves as a directory entry or documentation only. |

### 13.3 Capability consent flow

When `consent mode="capability"` is declared, the `consent` element shall contain the following sub-elements:

**`issue` element:**

| Attribute | Required | Description |
|---|---|---|
| `endpoint` | Yes | Path of the token issuance endpoint, relative to the document base URL. |
| `method` | Yes | HTTP method for the issuance request. Shall be `POST`. |

**`present` element:**

| Attribute | Required | Description |
|---|---|---|
| `header` | Yes | Name of the HTTP request header used to present the token. Shall be `Authorization`. |
| `scheme` | Yes | Authorization scheme name. Shall be `W4-Capability`. |

**Capability consent interaction sequence:**

1. The accessing party sends a request to the `issue` endpoint using the declared HTTP method. The request body and any required authentication are determined by the hosting party's implementation.
2. The hosting party validates the request and, if approved, returns a capability token in the response body as a JSON object with a `token` field.
3. The accessing party invokes the service binding, including the header `Authorization: W4-Capability <token>`.
4. The Web4 server runtime validates the token before executing the binding.

NOTE — The issuance step may involve human interaction, business logic validation, or other out-of-band processes defined by the hosting party.

### 13.4 Capability token format

This Technical Specification does not mandate a specific format for capability tokens. Implementations should use JSON Web Tokens (JWT) as defined in RFC 7519.

When JWT is used, the token should include the following claims:

| Claim | Description |
|---|---|
| `iss` | Issuer: the canonical URL of the hosting party's W4 page or domain. |
| `sub` | Subject: an identifier for the accessing party. |
| `exp` | Expiration time: a short time-to-live is recommended. |
| `scope` | Authorized service identifiers (may be a space-separated list). |

Regardless of token format, a hosting party should support token revocation before expiry.

### 13.5 Mutual consent semantics

The `capability` mode implements mutual consent as follows:

- **Hosting party consent** is expressed by the act of issuing the token.
- **Accessing party consent** is expressed by the act of presenting the token when invoking the service.

---

## 14. Policy

### 14.1 Policy element

The `policy` element within a `service` provides declarative access-control and operational hints. It is informative for agents performing planning or compliance checks and normative for conformant Web4 server runtimes.

### 14.2 Defined policy fields

The following child elements of `policy` are defined for v0.1:

**`rateLimit`**

| Attribute | Required | Description |
|---|---|---|
| `value` | Yes | Rate limit expressed as `N/unit` where unit is `s` (second), `m` (minute), `h` (hour), or `d` (day). Example: `60/m`. |

A conformant Web4 server runtime shall enforce the declared rate limit and shall return HTTP 429 when the limit is exceeded.

**`allowOrigins`**

| Attribute | Required | Description |
|---|---|---|
| `value` | Yes | Allowed HTTP origin or `*` for all origins. |

**`allowAgents`**

| Attribute | Required | Description |
|---|---|---|
| `value` | Yes | Allowed agent identifier pattern or `*` for all agents. |

**`costHint`**

| Attribute | Required | Description |
|---|---|---|
| `latencyMs` | No | Estimated typical latency in milliseconds. |
| `price` | No | Cost per invocation as a string (e.g., `"0"`, `"0.001 USD"`). |

`costHint` is informative only and shall not be enforced by the server runtime.

**`sandbox`**

| Attribute | Required | Description |
|---|---|---|
| `value` | Yes | `true` if the service should be executed in an isolated sandbox environment; `false` otherwise. |

`sandbox` applies primarily to local bindings. A conformant runtime should honour this declaration when sandboxing capability is available.

Extension policy elements in non-default namespaces may appear within `policy` and shall be ignored by parsers that do not recognize them.

---

## 15. Bindings

### 15.1 External binding descriptor (`.w4s`)

Bindings are declared in an external descriptor file with extension `.w4s`, referenced by `service/@sourceRef`.
The same `.w4s` path also serves as the public invocation endpoint for remote accessing parties.

A `.w4s` document shall be UTF-8 encoded W4ML subset XML with root element `w4s` in namespace `urn:w4ml:0.1`.

| Attribute/Element | Required | Description |
|---|---|---|
| `w4s/@service` | No | Service id. If present, it shall equal the referencing `service/@id`. |
| `w4s/bindings` | Yes | Container for one or more `binding` elements. |
| `w4s/bindings/binding` | Yes | Binding descriptors using the attributes defined in Clauses 15.3 and 15.4. |

Inline `<bindings>` inside `service` are not part of this version of the Core.

EXAMPLE (`services/math.add.w4s`)

```xml
<w4s xmlns="urn:w4ml:0.1" service="math.add">
  <bindings>
    <binding type="http" method="POST" endpoint="/rpc/math.add"
             contentType="application/json"/>
  </bindings>
</w4s>
```

### 15.2 Invocation endpoint and binding selection

The accessing party shall invoke a service by sending the request to the `.w4s` path declared by `service/@sourceRef`.
The hosting runtime then resolves and executes one binding descriptor from that `.w4s` document.
When multiple binding descriptors are present in `.w4s`, they declare alternative execution paths. A conformant runtime shall select the most appropriate binding based on runtime capabilities and policy.

### 15.3 HTTP binding

An HTTP binding declares invocation via an HTTP endpoint.

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `type` | Yes | Shall be `http`. |
| `method` | Yes | HTTP method: `GET`, `POST`, `PUT`, `DELETE`, or `PATCH`. |
| `endpoint` | Yes | Path relative to the document base URL, or an absolute URI. |
| `contentType` | No | MIME type of the request body. Default is `application/json`. |

**HTTP invocation semantics:**

a) When an `input` element is declared, the request body shall be a JSON object conforming to the declared input schema.
b) When no `input` element is declared, the request body may be omitted. If present, it shall be an empty JSON object.
c) When an `output` element is declared, the success response body shall be a JSON object conforming to the declared output schema.
d) When no `output` element is declared, the runtime may return HTTP `204` with no response body, or HTTP `200` with an empty JSON object.
e) On error, the response shall conform to the error structure defined in Clause 16.

### 15.4 Local binding

A local binding declares invocation via a local runtime entrypoint. Local bindings shall not be exposed to remote accessing parties; they are resolved and executed solely within the hosting party's runtime environment.

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `type` | Yes | Shall be `local`. |
| `exec` | Yes | Entrypoint reference in the form `runtime:entrypoint` (e.g., `python:module.function`, `bin:/usr/local/bin/tool`). |

---

## 16. Error Model

### 16.1 Runtime error structure

When a service invocation fails, the Web4 server runtime shall return a JSON response body conforming to the following structure:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable description.",
    "retryable": false,
    "details": {}
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `code` | string | Yes | Machine-readable error code from the enumeration below or an implementation-defined extension code. |
| `message` | string | Yes | Human-readable error description. |
| `retryable` | boolean | Yes | `true` if the client may retry the request; `false` if retry would not succeed without change. |
| `details` | object | No | Additional error-specific information. |

### 16.2 Standard error codes

| Code | HTTP Status | Description |
|---|---|---|
| `INVALID_ARGUMENT` | 400 | The input does not conform to the declared input schema. |
| `UNAUTHORIZED` | 401 | No valid capability token was presented. |
| `FORBIDDEN` | 403 | The capability token is valid but does not authorize this invocation. |
| `NOT_FOUND` | 404 | The requested service or resource does not exist. |
| `CONSENT_REQUIRED` | 403 | Invocation requires consent that has not been obtained. |
| `RATE_LIMITED` | 429 | The rate limit declared in policy has been exceeded. |
| `EFFECTS_BLOCKED` | 403 | The effects level requires consent that has not been satisfied. |
| `INTERNAL_ERROR` | 500 | An unexpected error occurred within the runtime. |

### 16.3 Error declarations in W4ML

A `service` may optionally declare the set of error codes it can return using the `errors` element:

```xml
<errors>
  <error code="INVALID_ARGUMENT" retryable="false"/>
  <error code="RATE_LIMITED"     retryable="true"/>
</errors>
```

Error declarations are informative and assist agents in planning retry strategies.

---

## 17. Progressive Loading

### 17.1 Load attribute

The `load` attribute may be placed on any `section` or `service` element to declare its loading strategy.

| Value | Description |
|---|---|
| `eager` | The element's content is included in the initial load. This is the default when `load` is absent. |
| `lazy` | The element is deferred; only its `id`, `name`, and summary metadata are retained until explicitly requested. |
| `never` | The element is not loaded unless explicitly requested by a direct fragment address. |

### 17.2 Agent-side loading semantics

A conformant Web4 agent runtime shall implement the following loading semantics:

a) `eager` sections shall be included in the agent's active knowledge context upon initial document load.
b) `lazy` sections shall contribute only their `id` and `name` to the initial context; the agent runtime shall fetch the full section content when it becomes relevant by mapping `#section=<id>` to the HTTP selector defined in Clause 20.2.
c) `eager` services shall be registered in the agent's active tool list upon initial document load.
d) `lazy` services shall be added to a candidate tool set but shall not appear in the active tool list until explicitly activated; upon activation, the agent runtime shall fetch the service definition by mapping `#service=<id>` to the HTTP selector defined in Clause 20.2.
e) `never` elements shall not be loaded or registered unless directly addressed by an explicit fragment address request.

### 17.3 Server-side loading semantics

A conformant Web4 server runtime shall support fragment-addressed retrieval of individual sections and services, as defined in Clause 20. When an HTTP fragment selector request is received, the runtime shall return only the addressed element and any types it references.

---

## 18. HTTP(S) Transport and Content Negotiation

### 18.1 Content negotiation

A conformant Web4 server runtime shall support HTTP content negotiation per RFC 9110, Section 12 on all URLs that serve W4ML documents.

The following media types are defined for Web4 representations:

| Accept header value | Response content | Requirement |
|---|---|---|
| `application/w4ml+xml` | The raw W4ML source document. | Required |
| `text/html` | An HTML-rendered view of the document (see Clause 24). | Required |
| `application/w4+json` | The compiled tool JSON view (see Clause 22). | Optional, but recommended |

When a client sends a request with `Accept: */*` or without an `Accept` header, the server should return the HTML view.

### 18.2 Required response headers

When responding to a content-negotiated request, the server runtime shall include:

```
Vary: Accept
```

When returning the HTML view, the server should include a `Link` header identifying the canonical W4ML source:

```
Link: <https://example.com/index.w4>; rel="canonical"; type="application/w4ml+xml"
```

### 18.3 Transport security

All production deployments of Web4 server runtimes should use HTTPS (HTTP over TLS as defined in RFC 9112 with TLS 1.2 or later). HTTP without TLS should be used only in development environments.

The `id` attribute of a W4 page should use an `https://` URI.

### 18.4 CORS

The `allowOrigins` policy declaration (Clause 14) should be reflected by the server runtime in the `Access-Control-Allow-Origin` response header for cross-origin requests.

---

## 19. Discovery and Linking

### 19.1 Well-known entry paths

A Web4 deployment should expose its root W4 page at the following path:

```
/index.w4
```

For sub-path namespaces, the entry page should be located at:

```
/<subpath>/index.w4
```

This convention enables agents and crawlers to discover W4 pages without prior knowledge of a site's URL structure.

### 19.2 HTML link integration

When a site exposes an HTML home page, it should include the following element in the HTML `<head>` to declare the associated W4 entry page:

```html
<link rel="w4" href="/index.w4" type="application/w4ml+xml">
```

This mechanism allows discovery of Web4 content from existing Web2 infrastructure.

### 19.3 W4ML link element

Within the W4ML `head` element, `link` elements declare relationships between W4 pages and external resources.

**Attributes:**

| Attribute | Required | Description |
|---|---|---|
| `rel` | Yes | Relationship type (see Clause 19.4). |
| `href` | Yes | URI of the linked resource, per RFC 3986. |
| `type` | No | MIME type of the linked resource. |

### 19.4 Link relation types

The following `rel` values are defined for v0.1:

| `rel` value | Description |
|---|---|
| `canonical` | The canonical URI of this W4 page. Should match the `id` attribute of the root `w4` element. |
| `template` | URI of the HTML template used to render this page (see Clause 24). |
| `import` | URI of another W4 page whose schema and services are imported into this document's context. |
| `see-also` | URI of a related W4 page, for informational cross-referencing. |
| `peer` | URI of a peer agent's W4 page exposing an `agent` service. Used for agent-to-agent discovery. |

A conformant Web4 agent runtime shall resolve `import` links and make the imported types and services available within the current document's context.

---

## 20. Fragment Addressing

### 20.1 Fragment syntax

A fragment address is a URI fragment component appended to a W4 page URI to identify a specific element within that document.

The fragment component shall conform to one of the following forms:

| Fragment form | Identifies |
|---|---|
| `#section=<id>` | The `section` element with the matching `id` attribute. |
| `#service=<id>` | The `service` element with the matching `id` attribute. |

The `<id>` part shall be percent-encoded per RFC 3986, Section 2.1, when the identifier contains characters outside the unreserved character set.

EXAMPLE

```
https://example.com/index.w4#section=vision
https://example.com/index.w4#service=math.add
```

### 20.2 Fragment resolution

A conformant Web4 server runtime shall resolve fragment-targeted retrieval through an HTTP fragment selector mapping.

At minimum, the runtime shall support the query parameter `w4fragment` with values in one of the following forms:

- `section:<id>`
- `service:<id>`

For a valid selector, the runtime shall:

a) parse the selector to extract the element type (`section` or `service`) and target `id`;
b) locate the addressed element within the W4ML document;
c) return a W4ML response containing only the addressed element, together with any `type` definitions it references.

A conformant Web4 agent runtime shall use this selector mapping to implement lazy loading retrieval as specified in Clause 17.2.

If the selector is syntactically invalid, the server shall return HTTP 400 with an error body conforming to Clause 16 and error code `INVALID_ARGUMENT`.
If the addressed element does not exist in the document, the server shall return HTTP 404 with an error body conforming to Clause 16 and error code `NOT_FOUND`.

### 20.3 Fragment addressing and HTTP

HTTP does not transmit URI fragment components to the server; fragment resolution is normally a client-side operation in web browsers. In Web4, fragment addresses defined in Clause 20.1 are logical identifiers. Clients shall map those logical fragment addresses to the HTTP fragment selector defined in Clause 20.2 (for example, `#section=<id>` maps to `?w4fragment=section:<id>`).

A server runtime may additionally support equivalent path-based mappings, provided semantics remain consistent with Clause 20.2.

---

## 21. Agent Communication

### 21.1 Agent service declaration

A service with `kind="agent"` declares an agent-to-agent communication endpoint. The `kind="agent"` service follows all rules defined for `kind="tool"` services and additionally conveys the semantics of a conversational or multi-turn interaction.

EXAMPLE

```xml
<service id="agent.chat" name="chat" kind="agent" load="eager"
         sourceRef="services/agent.chat.w4s">
  <intent lang="en">Engage in structured dialogue with this agent.</intent>
  <input  typeRef="ChatInput"/>
  <output typeRef="ChatOutput"/>
  <effects level="read"/>
  <consent mode="capability">
    <issue   endpoint="/consent/issue" method="POST"/>
    <present header="Authorization"    scheme="W4-Capability"/>
  </consent>
  <policy>
    <rateLimit value="20/m"/>
  </policy>
</service>
```

### 21.2 Standard message schema

This Technical Specification defines two standard schema types for agent communication. Implementations should use these types for interoperability; they may extend them with additional properties in non-conflicting property names.

**ChatInput:**

```xml
<type id="ChatInput" kind="object">
  <property name="thread_id"   type="string"  required="false"
            desc="Session identifier for multi-turn conversation."/>
  <property name="message"     type="string"  required="true"
            desc="The message content from the accessing party."/>
  <property name="context_ref" type="uri"     required="false"
            desc="URI of an external W4 page or fragment to include as context."/>
</type>
```

**ChatOutput:**

```xml
<type id="ChatOutput" kind="object">
  <property name="thread_id"       type="string"  required="true"
            desc="Session identifier, echoed or newly assigned."/>
  <property name="reply"           type="string"  required="true"
            desc="The reply from the hosting agent."/>
  <property name="tool_suggestions" type="array"  required="false"
            desc="Optional list of service IDs the accessing party may wish to invoke."/>
</type>
```

### 21.3 Session management

Multi-turn sessions shall be identified by a `thread_id` value. The hosting party shall assign a `thread_id` on the first turn and return it in `ChatOutput`. The accessing party shall include the same `thread_id` in subsequent turns within the same session.

The hosting party may expire sessions. When an expired `thread_id` is presented, the hosting party shall return an `INVALID_ARGUMENT` error with a message indicating session expiry.

### 21.4 Agent discovery via peer links

An agent publishes its communication entry point by exposing a service with `kind="agent"` in its W4 page. Other W4 pages discover this agent by including a `link rel="peer"` element in their `head`:

```xml
<link rel="peer" href="https://agent-b.example.com/index.w4"
      type="application/w4ml+xml"/>
```

Crawlers and agent runtimes may traverse `peer` links to build a graph of reachable agents.

---

## 22. Interoperability

### 22.1 Internal normalized model (implementation-defined)

A conformant W4ML parser shall parse and validate a W4 document so that Clause 22.2 compilation can be executed. An implementation may use an internal normalized model, and this Technical Specification does not require exposing that internal model as an interoperability target.

### 22.2 OpenAI-like tool JSON compilation

A conformant compiler shall generate an OpenAI-like tool JSON object for each `service` with `kind="tool"` or `kind="agent"` as follows:

```json
{
  "type": "function",
  "function": {
    "name":        "<service id>",
    "description": "<primary intent text>",
    "parameters":  { /* JSON Schema object compiled from input typeRef */ }
  },
  "x-w4": {
    "id":       "<canonical URI>#service=<service id>",
    "effects":  "<effects level>",
    "consent":  { "mode": "<consent mode>" },
    "bindings": [ { "type": "http", "endpoint": "/services/<service>.w4s" } ],
    "load":     "<load strategy>"
  }
}
```

The `x-w4` extension object preserves Web4 semantics for runtimes that understand it, while the `function` object provides compatibility with OpenAI-style tool use APIs.

**Compilation rules:**

a) The `name` field shall be the service `id` attribute value.
b) The `description` field shall be the text content of the `intent` element with the best language match, or the first `intent` if no language match is found.
c) The `parameters` field shall be the JSON Schema object produced by compiling the input `typeRef` per Clause 10.6.
d) The `x-w4.id` field shall be the canonical URI of the document concatenated with the service fragment address.

### 22.3 Naming conventions for compiled tools

a) The document canonical URI (`w4/@id`) shall be an HTTPS URL.
b) `section` `id` values shall match the pattern `[a-z][a-z0-9_-]*`.
c) `service` `id` values shall match the pattern `[a-z][a-z0-9._-]*`.
d) `property` `name` values shall match the pattern `[a-z_][a-z0-9_]*`.

When multiple W4 documents are aggregated in a single runtime and `service` `id` values conflict, the runtime shall prefix tool names using the form `w4__{hostname}__{service_id}`.

### 22.4 MCP conceptual relationship

MCP may be regarded as a service-only subset of Web4 with no knowledge-bearing content layer and no network-level discovery. W4ML is a superset in that it unifies services and content within a discoverable network graph. This Technical Specification does not define a normative compilation target for MCP; compatibility layers are implementation-defined.

---

## 23. Security Model

### 23.1 Trust boundaries

a) The text content of `section` elements is information. A conformant Web4 agent runtime shall treat section content as data that may influence the agent's knowledge or recommendations, but shall not allow section content to directly trigger service invocations or bypass consent, policy, or effects enforcement.
b) Service definitions (`service` elements) are trusted declarations from the hosting party. An agent runtime shall validate service definitions against this Technical Specification before using them.
c) Imported schemas and services (via `link rel="import"`) carry the trust level of their source URI. An agent runtime should apply the same effects and consent enforcement to imported services as to locally declared services.

### 23.2 Default invocation policy

The default invocation policy of a conformant Web4 agent runtime, absent any user- or deployment-level override, shall be:

| Effects level | Consent mode | Default action |
|---|---|---|
| `none` | `open` | May invoke automatically. |
| `read` | `open` | May invoke automatically. |
| `none` | `capability` | Shall obtain capability token; may then invoke. |
| `read` | `capability` | Shall obtain capability token; may then invoke. |
| `write` | any | Shall require explicit consent before invoking. |
| `control` | any | Shall require explicit consent before invoking. |
| `financial` | any | Shall require explicit consent before invoking. |
| `unknown` | any | Shall not invoke automatically; shall treat as requiring explicit consent. |

### 23.3 Prompt injection defence

A conformant Web4 agent runtime should implement safeguards against prompt injection via section content. In particular, the runtime should not execute instructions embedded as natural language within section text that attempt to escalate effects levels, suppress consent checks, or modify policy declarations.

### 23.4 Capability token security

a) Capability tokens should have a short time-to-live (`exp` claim).
b) The hosting party should maintain a revocation list for issued tokens.
c) Capability tokens shall be transmitted only over HTTPS.
d) The scope of a capability token should be limited to the specific service(s) for which it was issued.

### 23.5 Rate limit enforcement

A conformant Web4 server runtime shall enforce the `rateLimit` policy field. When a request exceeds the declared rate limit, the runtime shall return HTTP 429 with an error body conforming to Clause 16 and an error code of `RATE_LIMITED`.

---

## 24. Human Rendering

### 24.1 Rendering contract

When a Web4 server runtime returns an HTML response (per Clause 18.1), it shall satisfy the following minimum rendering contract:

a) All `section` elements with `load="eager"` or `load` absent shall be rendered as visible content in the HTML document.
b) All `service` elements with `load="eager"` or `load` absent should be rendered as human-readable service descriptions, including at minimum the service name and intent text.
c) The rendered HTML should preserve the reading order of elements as they appear in the `body` of the W4ML document.

The server runtime may render `load="lazy"` and `load="never"` elements as collapsed, hidden, or omitted from the initial HTML view.

### 24.2 Template association

A W4ML document may declare an HTML rendering template by including a `link rel="template"` element in `head`:

```xml
<link rel="template" href="https://example.com/templates/default.html"
      type="text/html"/>
```

When a template is declared, the server runtime should use the referenced template to render the HTML response. The template technology (server-side rendering, static generation, etc.) is implementation-defined.

### 24.3 Class-based styling

The `class` attribute on `section` and `service` elements provides styling hooks for templates. Class values are space-separated tokens and follow the same semantics as HTML `class` attributes. The server runtime should apply template-defined styles associated with these class values during HTML rendering.

### 24.4 Default template

When no `link rel="template"` is declared, the server runtime shall use a built-in default template. The default template shall satisfy the rendering contract of Clause 24.1. The visual design of the default template is implementation-defined.

---

## 25. Extensibility

### 25.1 Extension namespaces

W4ML supports extension via XML namespaces. Extension namespaces shall be declared using `xmlns:prefix` attributes. Extension elements and attributes in non-default namespaces may appear in any position allowed by the grammar in Annex A.

### 25.2 Extension element handling

A conformant W4ML parser:

a) shall not reject a document containing extension elements in non-default namespaces;
b) shall preserve the tag name, attributes, and raw text content of extension elements;
c) shall not apply the semantics of core elements to extension elements, even if their local names coincide with core element names;
d) shall not propagate security, consent, or effects decisions based on unrecognized extension elements.

### 25.3 Extension attribute handling

A conformant W4ML parser shall silently ignore unrecognized attributes on core elements. Unrecognized attributes shall not affect parsing of recognized attributes on the same element.

### 25.4 Profiles

A Profile is a standardized extension package identified by a URN. Profiles add domain-specific semantics (element types, attribute values, binding types) on top of the Core defined by this Technical Specification.

Profile URNs shall follow the form:

```
urn:w4ml:profile:<name>:<version>
```

EXAMPLE

```
urn:w4ml:profile:iot:0.1
urn:w4ml:profile:stream:0.1
```

Profiles shall not redefine the semantics of Core elements or override Core security decisions unless the overriding Profile is explicitly recognized and enabled by the runtime.

This Technical Specification defines no Profiles. Profiles are reserved for future editions.

### 25.5 Extension constraints

Extensions shall not:

a) redefine the meaning of the `effects` enumeration values defined in Clause 12.2;
b) introduce new consent modes that reduce the protection afforded by the security model of Clause 23;
c) alter the fragment addressing syntax defined in Clause 20.

---

## Annex A (normative) — W4ML Formal Grammar

This annex defines the formal grammar of W4ML in Extended Backus–Naur Form (EBNF). The notation used is:

| Notation | Meaning |
|---|---|
| `::=` | Production rule definition |
| `\|` | Alternative |
| `( )` | Grouping |
| `?` | Zero or one occurrence |
| `*` | Zero or more occurrences |
| `+` | One or more occurrences |
| `"..."` | Literal string |
| `[...]` | Character class |
| `#xNN` | Unicode code point (hexadecimal) |
| `(* ... *)` | Comment |

```ebnf
(* ============================================================ *)
(* W4ML Grammar v0.1                                            *)
(* ============================================================ *)

(* --- Top-level document --- *)

Document      ::= XMLDecl? S? W4Element S?

XMLDecl       ::= "<?xml" S "version" Eq ( '"1.0"' | '"1.1"' )
                  ( S "encoding" Eq ( '"UTF-8"' | '"utf-8"' ) )?
                  S? "?>"

(* --- Root element --- *)

W4Element     ::= "<w4"
                    ( S W4OptionalAttr )*
                    S W4CoreAttrs
                    ( S W4OptionalAttr )* S? ">"
                    S? HeadElement
                    S? BodyElement
                    S?
                  "</w4>"

W4CoreAttrs   ::= XmlnsAttr S VersionAttr
                | VersionAttr S XmlnsAttr

W4OptionalAttr ::= IdAttr | LangAttr | ExtAttr

XmlnsAttr     ::= "xmlns" Eq ( '"urn:w4ml:0.1"' | "'urn:w4ml:0.1'" )
VersionAttr   ::= "version" Eq ( '"0.1"' | "'0.1'" )
IdAttr        ::= "id" Eq QuotedURIRef
LangAttr      ::= "lang" Eq QuotedLangTag

(* --- Head element --- *)

HeadElement   ::= "<head>" S? HeadContent "</head>"
HeadContent   ::= ( TitleElement | MetaElement | LinkElement | ExtElement | Comment | S )*

TitleElement  ::= "<title" ( S LangAttr )? S? ">" CharData "</title>"

MetaElement   ::= "<meta"
                    S "name"    Eq QuotedString
                    S "content" Eq QuotedString
                    ( S ExtAttr )*
                  S? "/>"

LinkElement   ::= "<link"
                    S RelAttr
                    S HrefAttr
                    ( S TypeAttr )?
                    ( S ExtAttr )*
                  S? "/>"

RelAttr       ::= "rel"  Eq QuotedString
HrefAttr      ::= "href" Eq QuotedURIRef
TypeAttr      ::= "type" Eq QuotedMIMEType

(* --- Body element --- *)

BodyElement   ::= "<body>" S? BodyContent "</body>"
BodyContent   ::= ( SectionElement | SchemaElement | ServiceElement | ExtElement | Comment | S )*

(* --- Section element --- *)

SectionElement ::= "<section"
                     S "id" Eq QuotedName
                     ( S "name"  Eq QuotedString )?
                     ( S ClassAttr )?
                     ( S LoadAttr )?
                     ( S ExtAttr )*
                   S? ">"
                     S? SectionContent
                   "</section>"

SectionContent ::= ( W4LiteElement | ExtElement | CharData | Comment | S )*

(* --- W4-lite block elements --- *)

W4LiteElement ::= PElement | UlElement | OlElement | PreElement
                | AnchorElement | StrongElement | EmElement | BrElement

PElement      ::= "<p" ( S ClassAttr )? ( S ExtAttr )* S? ">"
                    InlineContent
                  PClose

PClose        ::= "</p>"
                  (* Parsers MAY infer </p> immediately before
                     <p>, <ul>, <ol>, <pre>, or </section>      *)

UlElement     ::= "<ul" ( S ExtAttr )* S? ">" S? LiContent "</ul>"
OlElement     ::= "<ol" ( S ExtAttr )* S? ">" S? LiContent "</ol>"
LiContent     ::= ( LiElement | Comment | S )*

LiElement     ::= "<li" ( S ExtAttr )* S? ">"
                    InlineContent
                  LiClose

LiClose       ::= "</li>"
                  (* Parsers MAY infer </li> immediately before
                     <li>, </ul>, or </ol>                       *)

PreElement    ::= "<pre" ( S ExtAttr )* S? ">" PreContent "</pre>"
PreContent    ::= ( [^<] | CDataSection )*
                  (* Treated as literal text; no child elements  *)

(* --- W4-lite inline elements --- *)

InlineContent ::= ( CodeElement | AnchorElement | StrongElement
                  | EmElement   | BrElement     | CharData       )*

CodeElement   ::= "<code" ( S ExtAttr )* S? ">" CharData CodeClose
CodeClose     ::= "</code>"
                  (* Parsers MAY infer </code> before
                     </p>, </li>, or end of containing element   *)

AnchorElement ::= "<a" S HrefAttr ( S ExtAttr )* S? ">"
                    InlineContent
                  "</a>"

StrongElement ::= "<strong>" InlineContent "</strong>"
EmElement     ::= "<em>"     InlineContent "</em>"
BrElement     ::= "<br" S? "/>"

(* --- Schema element --- *)

SchemaElement ::= "<schema>" S? TypeDefinition* S? "</schema>"

TypeDefinition ::= "<type"
                     S "id"   Eq QuotedName
                     S "kind" Eq QuotedTypeKind
                     ( S ExtAttr )*
                   S? ">"
                     S? TypeContent
                   "</type>"

TypeContent   ::= ( PropertyElement | AdditionalPropertiesElement | ExtElement | Comment | S )*

PropertyElement ::= "<property"
                      S "name" Eq QuotedName
                      S "type" Eq QuotedTypeNameOrRef
                      ( S "required"   Eq QuotedBool   )?
                      ( S "desc"       Eq QuotedString  )?
                      ( S "default"    Eq QuotedString  )?
                      ( S "min"        Eq QuotedNumber  )?
                      ( S "max"        Eq QuotedNumber  )?
                      ( S "minLength"  Eq QuotedNonNegInt )?
                      ( S "maxLength"  Eq QuotedNonNegInt )?
                      ( S "format"     Eq QuotedString  )?
                      ( S "enum"       Eq QuotedString  )?
                      ( S ExtAttr )*
                    S? "/>"

AdditionalPropertiesElement ::= "<additionalProperties"
                                  S "value" Eq QuotedBool
                                  ( S ExtAttr )*
                                S? "/>"

(* --- Service element --- *)

ServiceElement ::= "<service"
                     S "id"    Eq QuotedDotName
                     S "sourceRef" Eq QuotedW4SPath
                     ( S "name"  Eq QuotedString      )?
                     ( S "kind"  Eq QuotedServiceKind )?
                     ( S LoadAttr )?
                     ( S ClassAttr )?
                     ( S ExtAttr )*
                   S? ">"
                     S? ServiceContent
                   "</service>"

ServiceContent ::= ( IntentElement   | InputElement   | OutputElement
                   | EffectsElement  | ConsentElement
                   | PolicyElement   | ErrorsElement  | ExamplesElement
                   | ExtElement      | Comment        | S              )*

IntentElement  ::= "<intent" ( S LangAttr )? ( S ExtAttr )* S? ">"
                     CharData
                   "</intent>"

InputElement   ::= "<input"  S "typeRef" Eq QuotedName ( S ExtAttr )* S? "/>"
OutputElement  ::= "<output" S "typeRef" Eq QuotedName ( S ExtAttr )* S? "/>"

EffectsElement ::= "<effects" S "level" Eq QuotedEffectsLevel
                   ( S ExtAttr )* S? "/>"

ConsentElement ::= "<consent" S "mode" Eq QuotedConsentMode
                   ( S ExtAttr )* S? "/>"
                 | "<consent" S "mode" Eq QuotedConsentMode
                   ( S ExtAttr )* S? ">"
                     S? ( IssueElement | PresentElement | Comment | S )*
                   "</consent>"

IssueElement   ::= "<issue"
                     S "endpoint" Eq QuotedPath
                     S "method"   Eq QuotedPOSTMethod
                     ( S ExtAttr )*
                   S? "/>"

PresentElement ::= "<present"
                     S "header" Eq QuotedString
                     S "scheme" Eq QuotedString
                     ( S ExtAttr )*
                   S? "/>"

(* --- W4S external binding descriptor --- *)

W4sDocument   ::= "<w4s"
                    S "xmlns" Eq '"urn:w4ml:0.1"'
                    ( S "service" Eq QuotedDotName )?
                    ( S ExtAttr )*
                  S? ">"
                    S? BindingsElement
                  "</w4s>"

BindingsElement ::= "<bindings>" S? ( BindingElement | Comment | S )* "</bindings>"
BindingElement  ::= HttpBindingElement | LocalBindingElement

HttpBindingElement ::= "<binding"
                         S "type"        Eq QuotedHttpBindingType
                         S "method"      Eq QuotedHTTPMethod
                         S "endpoint"    Eq QuotedURIRef
                         ( S "contentType" Eq QuotedMIMEType )?
                         ( S ExtAttr )*
                       S? "/>"

LocalBindingElement ::= "<binding"
                          S "type" Eq QuotedLocalBindingType
                          S "exec" Eq QuotedExecRef
                          ( S ExtAttr )*
                        S? "/>"

PolicyElement  ::= "<policy>" S? PolicyContent "</policy>"
PolicyContent  ::= ( RateLimitElement   | AllowOriginsElement
                   | AllowAgentsElement | CostHintElement
                   | SandboxElement     | ExtElement
                   | Comment            | S               )*

RateLimitElement    ::= "<rateLimit"    S "value" Eq QuotedRateSpec    ( S ExtAttr )* S? "/>"
AllowOriginsElement ::= "<allowOrigins" S "value" Eq QuotedString      ( S ExtAttr )* S? "/>"
AllowAgentsElement  ::= "<allowAgents"  S "value" Eq QuotedString      ( S ExtAttr )* S? "/>"
CostHintElement     ::= "<costHint"
                          ( S "latencyMs" Eq QuotedNonNegInt )?
                          ( S "price"     Eq QuotedString    )?
                          ( S ExtAttr )*
                        S? "/>"
SandboxElement      ::= "<sandbox" S "value" Eq QuotedBool ( S ExtAttr )* S? "/>"

ErrorsElement  ::= "<errors>" S? ( ErrorDeclElement | Comment | S )* "</errors>"
ErrorDeclElement ::= "<error"
                       S "code"      Eq QuotedName
                       ( S "retryable" Eq QuotedBool )?
                       ( S ExtAttr )*
                     S? "/>"

ExamplesElement ::= "<examples>" S? ( ExampleElement | Comment | S )* "</examples>"
ExampleElement  ::= "<example>" S?
                      CallElement S? ResultElement
                    S? "</example>"
CallElement     ::= "<call>"   S? JSONText S? "</call>"
ResultElement   ::= "<result>" S? JSONText S? "</result>"

(* --- Extension elements --- *)

ExtElement    ::= "<" NSPrefix ":" NCName ( S ExtAttr )* S?
                  ( "/>"
                  | ">" ExtContent "</" NSPrefix ":" NCName ">"
                  )
ExtContent    ::= ( ExtElement | CharData | Comment | S )*

(* --- Shared attribute productions --- *)

ClassAttr     ::= "class" Eq QuotedClassList
LoadAttr      ::= "load"  Eq ( '"eager"' | '"lazy"' | '"never"'
                              | "'eager'" | "'lazy'" | "'never'" )
ExtAttr       ::= NSPrefix ":" NCName Eq QuotedString

(* --- Comments --- *)

Comment       ::= "<!--" ( Char* - ( Char* "-->" Char* ) ) "-->"

(* --- Quoted terminal productions --- *)
(* Both double-quote and single-quote delimiters are accepted.  *)
(* Only the double-quote form is shown; replace '"' with "'"    *)
(* and [^"] with [^'] for the single-quote variant.            *)

QuotedString       ::= '"' [^"]* '"'  |  "'" [^']* "'"
QuotedURIRef       ::= '"' URIRef '"' |  "'" URIRef "'"
QuotedPath         ::= '"' Path '"'   |  "'" Path "'"
QuotedW4SPath      ::= '"' W4SPath '"'|  "'" W4SPath "'"
QuotedName         ::= '"' Name '"'   |  "'" Name "'"
QuotedDotName      ::= '"' DotName '"'|  "'" DotName "'"
QuotedLangTag      ::= '"' LangTag '"'|  "'" LangTag "'"
QuotedMIMEType     ::= '"' MIMEType '"'| "'" MIMEType "'"
QuotedBool         ::= '"true"'  | '"false"'  | "'true'"  | "'false'"
QuotedNumber       ::= '"' Number '"'  |  "'" Number "'"
QuotedNonNegInt    ::= '"' NonNegInt '"'| "'" NonNegInt "'"
QuotedClassList    ::= '"' ClassList '"'| "'" ClassList "'"
QuotedRateSpec     ::= '"' RateSpec '"' | "'" RateSpec "'"
QuotedExecRef      ::= '"' ExecRef '"'  | "'" ExecRef "'"
QuotedTypeNameOrRef ::= '"' TypeNameOrRef '"' | "'" TypeNameOrRef "'"

QuotedEffectsLevel ::= '"none"'      | '"read"'    | '"write"'
                     | '"control"'   | '"financial"'| '"unknown"'
                     | "'none'"      | "'read'"    | "'write'"
                     | "'control'"   | "'financial'"| "'unknown'"

QuotedConsentMode  ::= '"open"'        | '"capability"'
                     | '"interactive"' | '"deny"'
                     | "'open'"        | "'capability'"
                     | "'interactive'" | "'deny'"

QuotedHTTPMethod   ::= '"GET"'  | '"POST"' | '"PUT"'
                     | '"DELETE"' | '"PATCH"'
                     | "'GET'"  | "'POST'" | "'PUT'"
                     | "'DELETE'" | "'PATCH'"

QuotedServiceKind  ::= '"tool"' | '"agent"' | "'tool'" | "'agent'"
QuotedPOSTMethod   ::= '"POST"' | "'POST'"
QuotedHttpBindingType ::= '"http"' | "'http'"
QuotedLocalBindingType ::= '"local"' | "'local'"

QuotedTypeKind     ::= '"object"'   | '"array"'   | '"string"'
                     | '"int"'      | '"float"'   | '"bool"'
                     | '"datetime"' | '"uri"'     | '"bytes"'
                     | "'object'"   | "'array'"   | "'string'"
                     | "'int'"      | "'float'"   | "'bool'"
                     | "'datetime'" | "'uri'"     | "'bytes'"

(* --- Lexical productions --- *)

Eq            ::= S? "=" S?
S             ::= ( #x20 | #x09 | #x0D | #x0A )+
Char          ::= #x09 | #x0A | #x0D | [#x20-#xD7FF]
                | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
CharData      ::= ( [^<&] | "&amp;" | "&lt;" | "&gt;"
                  | "&quot;" | "&apos;" | "&#x" HexDigits ";" )*
CDataSection  ::= "<![CDATA[" ( Char* - ( Char* "]]>" Char* ) ) "]]>"

Name          ::= NameStartChar NameChar*
NameStartChar ::= [a-z] | [A-Z] | "_"
NameChar      ::= NameStartChar | [0-9] | "-"
NCName        ::= NameStartChar NameChar*
NSPrefix      ::= NCName
DotName       ::= Name ( "." Name )*
TypeNameOrRef ::= Name ( "." Name )*   (* base type or user-defined type id *)
LangTag       ::= [a-z]{2,3} ( "-" [A-Za-z0-9]+ )*
ClassList     ::= NameChar+ ( " " NameChar+ )*
Number        ::= "-"? Digits ( "." Digits )?
Digits        ::= [0-9]+
NonNegInt     ::= [0-9]+
HexDigits     ::= [0-9A-Fa-f]+
RateSpec      ::= Digits "/" ( "s" | "m" | "h" | "d" )
ExecRef       ::= NCName ":" [!-~]+
                  (* Printable ASCII after the colon; e.g. python:module.func *)
URIRef        ::= (* URI or URI reference per RFC 3986 Section 4 *)
Path          ::= "/" [^\x00-\x1F\x7F\x20"'<>\\^`{|}]*
W4SPath       ::= [A-Za-z0-9._/-]+ ".w4s"
MIMEType      ::= (* Media type per RFC 6838 *)
JSONText      ::= (* JSON text per RFC 8259 *)
```

---

## Annex B (informative) — Complete W4ML Examples

### B.1 Minimal conformant W4 page

```xml
<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1"
    id="https://example.com/index.w4">

  <head>
    <title lang="en">Web4 Demo</title>
    <meta name="description" content="A minimal Web4 page."/>
    <link rel="canonical" href="https://example.com/index.w4"/>
    <link rel="template"  href="https://example.com/templates/default.html"
          type="text/html"/>
  </head>

  <body>

    <section id="intro" name="Introduction" load="eager">
      <p>This is a Web4 page. It carries both content and callable services.</p>
      <ul>
        <li>Structured knowledge for agents</li>
        <li>Callable tools in the same document</li>
        <li>Discoverable via standard HTTP</li>
      </ul>
    </section>

    <schema>
      <type id="AddInput" kind="object">
        <property name="a" type="int" required="true" desc="First operand."/>
        <property name="b" type="int" required="true" desc="Second operand."/>
        <additionalProperties value="false"/>
      </type>
      <type id="AddOutput" kind="object">
        <property name="sum" type="int" required="true"/>
      </type>
    </schema>

    <service id="math.add" name="add" kind="tool" load="eager"
             sourceRef="services/math.add.w4s">
      <intent lang="en">Add two integers and return their sum.</intent>
      <input  typeRef="AddInput"/>
      <output typeRef="AddOutput"/>
      <effects level="none"/>
      <consent mode="open"/>
      <policy>
        <rateLimit value="60/m"/>
        <allowOrigins value="*"/>
        <costHint latencyMs="5" price="0"/>
      </policy>
      <errors>
        <error code="INVALID_ARGUMENT" retryable="false"/>
        <error code="RATE_LIMITED"     retryable="true"/>
      </errors>
      <examples>
        <example>
          <call>{"a": 2, "b": 3}</call>
          <result>{"sum": 5}</result>
        </example>
      </examples>
    </service>

  </body>
</w4>
```

### B.2 Agent communication service with capability consent

```xml
<service id="agent.chat" name="chat" kind="agent" load="eager"
         sourceRef="services/agent.chat.w4s">
  <intent lang="en">Engage in structured multi-turn dialogue with this agent.</intent>
  <input  typeRef="ChatInput"/>
  <output typeRef="ChatOutput"/>
  <effects level="read"/>
  <consent mode="capability">
    <issue   endpoint="/consent/issue" method="POST"/>
    <present header="Authorization"    scheme="W4-Capability"/>
  </consent>
  <policy>
    <rateLimit value="20/m"/>
    <allowAgents value="*"/>
  </policy>
</service>
```

### B.3 Compiled OpenAI-like tool JSON for the math.add service

```json
{
  "type": "function",
  "function": {
    "name": "math.add",
    "description": "Add two integers and return their sum.",
    "parameters": {
      "type": "object",
      "properties": {
        "a": { "type": "integer", "description": "First operand." },
        "b": { "type": "integer", "description": "Second operand." }
      },
      "required": ["a", "b"],
      "additionalProperties": false
    }
  },
  "x-w4": {
    "id":      "https://example.com/index.w4#service=math.add",
    "effects": "none",
    "consent": { "mode": "open" },
    "bindings": [
      { "type": "http", "endpoint": "/services/math.add.w4s" }
    ],
    "load": "eager"
  }
}
```

### B.4 Discovery: HTML page linking to W4 entry

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>Example Site</title>
    <link rel="w4" href="/index.w4" type="application/w4ml+xml">
  </head>
  <body>
    <p>Visit our <a href="/">home page</a>.</p>
  </body>
</html>
```

---

## Annex C (informative) — Governance and License

### C.1 License

This Technical Specification is published under the **Apache License, Version 2.0**. The full license text is available at:

```
https://www.apache.org/licenses/LICENSE-2.0
```

Unless required by applicable law or agreed to in writing, material distributed under the Apache License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

### C.2 Contributions

Contributions to this Technical Specification are accepted under the terms of the Apache License, Version 2.0, unless another contributor license agreement is in effect.

### C.3 Versioning

This Technical Specification uses semantic versioning. The version `0.1` indicates a pre-release specification intended for implementation feedback. Backwards-incompatible changes will increment the minor version number during the pre-release phase.

---

*End of WEB4ORG-STD-0001 v0.1*
