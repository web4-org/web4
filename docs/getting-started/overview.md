# Overview

## What is Web4?

The contemporary Web is excellent at delivering content to human readers but provides weak
machine-readable semantics. Agent systems and tool protocols, conversely, excel at structured
invocation but lack the network-layer discoverability of the Web.

**Web4 bridges this gap.** A W4 page is a single HTTP-addressable resource that simultaneously:

- delivers **human-readable content** (sections of prose, lists, rich text)
- exposes **machine-invocable services** (typed tool calls with full schema and consent rules)
- is **agent-discoverable** through ordinary HTTP and linking mechanisms

A W4 page is authored in the **W4 Markup Language (W4ML)** — an XML-based format with file
extension `.w4` and MIME type `application/w4ml+xml`. The same URL can return different
representations depending on the `Accept` header: a rendered HTML page for a browser, a
structured JSON tool registry for an AI agent, or the raw W4ML document for a parser.

---

## Problem Statement

| Problem | Web4 Answer |
|---|---|
| Agents need discoverable, callable tools | Services declared inline in the document |
| Tools need typed inputs/outputs | W4Schema — a structured type system |
| Autonomous agents need authorization boundaries | Consent modes: `open`, `capability`, `interactive` |
| Services need access-control and rate limits | Per-service policy declarations |
| AI platforms need OpenAI-compatible tool lists | Compiler produces OpenAI-like tool JSON |
| Humans need a readable view of the same resource | Human rendering via Tera templates |

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                      W4 Page (.w4)                       │
│                                                          │
│  <head>  links, imports, peer refs, template             │
│  <body>                                                  │
│    <section>  human-readable content                     │
│    <schema>   shared type definitions (W4Schema)         │
│    <service>  callable capability + policy + consent     │
│  </body>                                                 │
└──────────────────┬───────────────────────────────────────┘
                   │ served over HTTP(S)
                   ▼
┌──────────────────────────────────────────────────────────┐
│                   web4-gateway (Axum)                    │
│                                                          │
│  W4mlParser ──► ParseResult (normalized internal model)  │
│       │                                                  │
│       ├─► Compiler ──► application/w4+json               │
│       ├─► Renderer ──► text/html  (Tera template)        │
│       └─► Raw      ──► application/w4ml+xml              │
│                                                          │
│  ServiceRuntime                                          │
│    ├─ Validator (JSON Schema)                            │
│    ├─ ConsentEngine (open / capability / interactive)    │
│    └─ BindingExecutor (HTTP / Local / Gateway)           │
└──────────────────────────────────────────────────────────┘
```

---

## Repository Layout

```
web4/
├── WEB4ORG-STD-0001.md          Normative specification
├── AGENTS.md                    Repository contributor guide
├── config.showcase.json         Example gateway config
│
├── crates/
│   ├── web4-core/               Parser, compiler, runtime library
│   │   └── src/
│   │       ├── w4ml.rs          W4ML parser and validator
│   │       ├── compiler.rs      W4ML → OpenAI-like tool JSON
│   │       ├── runtime/         Service execution pipeline
│   │       ├── api.rs           Public RuntimeOptions / build_default_runtime
│   │       ├── model.rs         Core data model types
│   │       ├── traits.rs        Extension traits
│   │       └── error.rs         Error types and HTTP mapping
│   │
│   └── web4-gateway/            HTTP gateway binary
│       └── src/
│           ├── main.rs          Entry point
│           ├── app.rs           Router construction + state init
│           ├── handlers.rs      HTTP handler functions
│           ├── renderer.rs      HTML rendering engine
│           ├── config.rs        Config loading
│           └── logic/           Auth, content, consent, policy layers
│
├── examples/
│   └── web4-root-showcase/      Runnable demo document
│       ├── showcase.w4          Main W4ML document
│       ├── services/            Service binding files (.w4s)
│       └── templates/           Custom Tera template
│
└── docs/                        ← you are here
```

---

## Key Concepts at a Glance

- **[W4 page](../concepts/web4-model.md)** — the unified document-plus-service resource
- **[W4ML](../concepts/w4ml-language.md)** — the XML-based markup language
- **[W4Schema](../reference/w4ml-syntax.md#schema)** — typed input/output declarations
- **[Consent modes](../concepts/consent-modes.md)** — `open`, `capability`, `interactive`
- **[Bindings](../concepts/bindings.md)** — HTTP, Local, Gateway execution mappings
- **[Content negotiation](../concepts/content-negotiation.md)** — same URL, three representations

---

## Specification

This implementation conforms to **WEB4ORG-STD-0001 v0.1**. See
[CONFORMANCE-MATRIX.md](../CONFORMANCE-MATRIX.md) for the clause-by-clause implementation status.
