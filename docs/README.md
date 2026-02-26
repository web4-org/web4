# Web4 Documentation

Welcome to the **web4** implementation documentation. This reference covers everything from
first-time setup to deep-dive internals, organized into five layers so you can jump straight
to what you need.

---

## Layers at a Glance

| Layer | Who it's for | Start here |
|---|---|---|
| [Getting Started](#getting-started) | New to the project | [Overview](getting-started/overview.md) |
| [Concepts](#concepts) | Want to understand the design | [Web4 Model](concepts/web4-model.md) |
| [Reference](#reference) | Need precise, complete specs | [W4ML Syntax](reference/w4ml-syntax.md) |
| [Guides](#guides) | Solving a specific task | [Authoring W4ML](guides/authoring-w4ml.md) |
| [Development](#development) | Contributing or embedding the library | [Contributing](development/contributing.md) |

---

## Getting Started

| Document | Description |
|---|---|
| [Overview](getting-started/overview.md) | What Web4 is, the problem it solves, architecture bird's-eye view |
| [Installation](getting-started/installation.md) | Prerequisites, building from source, verifying the install |
| [Quickstart](getting-started/quickstart.md) | Run the showcase in five minutes and try all three consent modes |

---

## Concepts

| Document | Description |
|---|---|
| [Web4 Model](concepts/web4-model.md) | W4 page anatomy: Document, Section, Schema, Service, W4 Graph |
| [W4ML Language](concepts/w4ml-language.md) | File structure, root element, head and body elements |
| [Consent Modes](concepts/consent-modes.md) | `open`, `capability`, and `interactive` — when and why |
| [Bindings](concepts/bindings.md) | HTTP, Local, and Gateway binding types |
| [Policy Enforcement](concepts/policy-enforcement.md) | Rate limiting, CORS, agent allowlists, effects levels |
| [Content Negotiation](concepts/content-negotiation.md) | The three `Accept` types, `Vary`, and `w4fragment` |

---

## Reference

| Document | Description |
|---|---|
| [W4ML Syntax](reference/w4ml-syntax.md) | Complete element and attribute reference |
| [Config Schema](reference/config-schema.md) | Every gateway JSON config field explained |
| [Error Codes](reference/error-codes.md) | Error code → HTTP status mapping table |
| **HTTP API** | |
| [Document Endpoints](reference/http-api/document-endpoints.md) | `GET /` and `GET /{source_ref}` |
| [Service Invoke](reference/http-api/service-invoke.md) | `POST /{source_ref}` |
| [Capabilities](reference/http-api/capabilities.md) | `POST /consent/issue` |
| [Challenges](reference/http-api/challenges.md) | `POST /consent/challenge` and approve/deny lifecycle |

---

## Guides

| Document | Description |
|---|---|
| [Authoring W4ML](guides/authoring-w4ml.md) | Writing `.w4` documents and `.w4s` service files |
| [Configuring the Gateway](guides/configuring-gateway.md) | Gateway config walkthrough with all options |
| [Custom Templates](guides/custom-templates.md) | Building Tera HTML templates for human rendering |
| [JWT Capabilities](guides/jwt-capabilities.md) | Issuing and presenting capability tokens |
| [Securing Services](guides/securing-services.md) | Production security best practices |
| [Agent Integration](guides/agent-integration.md) | How AI agents consume `application/w4+json` |

---

## Development

| Document | Description |
|---|---|
| [Contributing](development/contributing.md) | Code style, commit conventions, PR checklist |
| [Architecture](development/architecture.md) | Internal pipeline: Parser → Compiler → Runtime → Gateway |
| [Library API](development/library-api.md) | Using `web4-core` as an embedded Rust library |
| [Testing](development/testing.md) | Running and writing tests |

---

## Operational References (existing)

| Document | Description |
|---|---|
| [RUNBOOK.md](RUNBOOK.md) | Gateway startup, config quick-reference, library API surface |
| [CONFORMANCE-MATRIX.md](CONFORMANCE-MATRIX.md) | WEB4ORG-STD-0001 clause-by-clause implementation status |

---

## Normative Specification

The authoritative technical specification lives at the repository root:

**[WEB4ORG-STD-0001.md](../WEB4ORG-STD-0001.md)** — Web4 Core Technical Specification v0.1
