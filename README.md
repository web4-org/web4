# Web4

> *The Web solved discoverability. Tool protocols solved invocability. Nobody solved both.*
> Web4 is the attempt to do that — at web scale.

---

## Background: Three Partial Solutions to the Same Problem

We are building AI agents that need to perceive knowledge and invoke capabilities in the world.
The infrastructure we have for this is the result of decades of evolution aimed at a different
goal. None of it fits cleanly.

### Web Pages (HTML): Great Discoverability, Near-Zero Callability

The Web is the largest knowledge graph in human history. It is linkable, crawlable, addressable,
and universally accessible. These properties did not happen by accident — HTTP and URLs gave
every document a network identity, and hyperlinks wove them into a graph.

But the Web's fundamental semantic unit is a **styled box** (`<div>`). The information model
underneath HTML is about visual layout, not machine-interpretable intent. When an agent reads
a web page today, it is essentially doing OCR on structured text: scraping, heuristically
parsing, hoping the author's formatting conventions line up with its expectations.

More critically, **HTML has no model for callable capabilities**. There is no standard way to
declare "this page offers a function that takes these typed inputs and returns these typed
outputs." `<form>` comes closest, but it is a UI construct — no schema, no typed I/O contract,
no consent model, no effects declaration. Every web API that wants to be machine-callable has to
bolt on a separate REST/GraphQL/RPC layer, completely divorced from the page itself.

The Web is discoverable. It is not invocable.

### Tool Protocols (MCP, Function Calling, …): Great Callability, No Network

The agent-tool ecosystem learned the lesson: if you want agents to reliably call things, you
need typed interfaces. Function Calling, MCP, LangChain tools, and their peers are genuinely
well-designed for structured invocation — typed input schemas, typed output schemas, clear
error contracts.

But they made a foundational architectural choice, probably unavoidably, that turns out to be
very limiting: **tools are not web resources**. They have no URL. They live inside a runtime,
inside a framework, inside a process. There is no HTTP server you point a crawler at to
discover them.

This has cascading consequences:

- **No organic discovery.** You cannot find a tool unless the system you are in has been
  explicitly configured to know about it. There is no equivalent of "follow a link."
- **No cross-tool graph.** Tool A cannot naturally reference Tool B on a different server.
  The web's most powerful property — that any node can link to any other node — simply does
  not exist here.
- **No knowledge layer.** A tool definition is a pure function signature. It carries no
  prose, no context, no explanations that help an agent decide *whether* to call it and *how
  to use it well*. Knowledge and capability are split across two separate systems that never
  meet.
- **Fragmented ecosystem.** OpenAI's function calling, Anthropic's tool use, MCP, LangChain
  agents — each is a walled garden. A tool written for one does not naturally compose with
  another. Without a common network substrate, there is no force pushing toward
  interoperability.

Tool protocols are invocable. They are not a network.

### Agent Skills: Great Progressive Loading, Still a Local Island

Anthropic's Agent Skills represent a genuinely thoughtful design. A Skill is a filesystem
directory — a `SKILL.md` with metadata, supporting scripts, reference materials — and the
loading model is clever: metadata loads eagerly at startup (~100 tokens), full instructions
load only when the agent decides the Skill is relevant, and supporting files load only when
execution actually needs them. Many Skills can be installed with minimal context overhead.

This solves a real problem. But notice what the address of a Skill is: a file path. It lives
in a directory on the agent's VM. Its discovery radius is exactly the set of Skills that
someone has already installed in that specific environment.

Agent B, running on a different machine with different Skills installed, cannot discover
Agent A's Skills by following a link. There is no link to follow. Cross-agent composition
requires explicit out-of-band coordination — someone manually knowing that the other agent
exists, what it can do, and how to reach it. This is the same problem the pre-Web internet
had before URLs: capabilities existed, but you had to already know about them.

The progressive loading is elegant engineering. But progressive loading over a local registry
is not the same thing as a network. The boundary of an Agent Skill is the boundary of the VM
it lives in. That boundary does not dissolve, however cleverly the loading is staged.

Skills are sophisticated. They are not the web.

---

## The Structural Gap

Laid side by side, the pattern becomes clear:

|  | Network-addressable | Linkable / Discoverable | Callable | Typed I/O | Knowledge layer |
|---|:---:|:---:|:---:|:---:|:---:|
| Web page (HTML) | ✓ | ✓ | ✗ | ✗ | partial |
| Tool protocol (MCP…) | ✗ | ✗ | ✓ | ✓ | ✗ |
| Local skill / plugin | ✗ | partial | ✓ | partial | partial |
| **W4 page** | **✓** | **✓** | **✓** | **✓** | **✓** |

The Web already solved the hardest problem: universal addressing, linking, and discoverability
over HTTP are thirty-year-old solved problems. The insight behind Web4 is that you do not need
to invent a new network — you need to extend the web's document format to carry typed callable
services *alongside* content, and let HTTP do the rest.

---

## Web4: One Resource, Two Audiences

A **W4 page** is a single HTTP-addressable document that is simultaneously:

- a **content document** — sections of prose and structured knowledge, readable by humans and
  ingestable by agent context windows
- a **service registry** — typed, callable capabilities with schemas, consent modes, and
  access policy, ready for agent tool-use frameworks

The same URL serves both audiences through standard HTTP content negotiation:

```
GET https://example.com/page.w4
Accept: text/html              →  rendered HTML for a browser
Accept: application/w4+json   →  structured JSON tool list for an agent
Accept: application/w4ml+xml  →  raw W4ML source for a parser
```

No separate API surface. No out-of-band tool registry. No synchronization problem between
the "documentation site" and the "API spec." They are the same resource.

W4 pages are authored in **W4ML** — an XML-based markup language with extension `.w4`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="https://example.com/page.w4">
  <head>
    <title lang="en">Prime Number Tools</title>
    <link rel="peer" href="https://math.example.com/index.w4" type="application/w4ml+xml"/>
  </head>
  <body>

    <section id="intro" load="eager">
      A prime number is a natural number greater than 1 that has no positive divisors
      other than 1 and itself. This page exposes a tool to test primality.
    </section>

    <schema>
      <type id="CheckInput" kind="object">
        <property name="n" type="int" required="true" desc="Number to test"/>
        <additionalProperties value="false"/>
      </type>
      <type id="CheckOutput" kind="object">
        <property name="is_prime" type="bool" required="true"/>
        <additionalProperties value="false"/>
      </type>
    </schema>

    <service id="math.is_prime" name="Is Prime?" kind="tool" load="eager"
             sourceRef="services/is_prime.w4s">
      <intent lang="en">Returns whether n is a prime number.</intent>
      <input typeRef="CheckInput"/>
      <output typeRef="CheckOutput"/>
      <effects level="none"/>
      <consent mode="open"/>
      <policy><allowOrigins value="*"/></policy>
    </service>

  </body>
</w4>
```

An agent fetching this URL gets a compiled, OpenAI-compatible tool definition. A human gets
a readable HTML page. And because this page has a URL, it can be linked from any other W4 page,
crawled by any agent, and woven into the same graph as every other resource on the Web.

---

## Consent and Safety at the Protocol Level

Web4 does not assume all services should be freely invokable. Rather than treating
authorization as an application concern bolted on top, Web4 builds three consent modes into
the standard itself:

| Mode | Mechanism | Typical use |
|---|---|---|
| `open` | No authorization required | Public, side-effect-free computation |
| `capability` | Caller presents a signed JWT scoped to this service | Agent-specific or rate-sensitive access |
| `interactive` | A human must approve each invocation before it executes | Write operations, irreversible actions |

Services also declare a **effects level** (`none` / `read` / `write` / `admin`) and carry
per-service **policy** (rate limiting, CORS, agent allowlists). An agent can read all of these
*before* deciding to invoke — building a planning layer that understands consequences.

---

## This Repository

Reference implementation of Web4 v0.1, written in Rust.

```
web4/
├── WEB4ORG-STD-0001.md       Normative specification (Apache 2.0)
├── crates/
│   ├── web4-core/            Library: W4ML parser · compiler · service runtime
│   └── web4-gateway/         Binary: HTTP gateway (Axum) · rendering · auth · policy
├── examples/
│   └── web4-root-showcase/   Runnable demo — all three consent modes in one page
└── docs/                     Full documentation
```

**What is implemented (Phase A–H of the spec):**

- W4ML parser, semantic validator, and normalized internal model
- W4Schema type system → JSON Schema compilation and strict validation
- Content negotiation: `w4ml+xml` / `w4+json` / `text/html` with `Vary: Accept`
- `w4fragment` selector for section and service addressing
- Full service invocation pipeline: effects gate → consent gate → policy → binding → I/O validation
- HTTP binding executor and local process binding (`bin:` / `mcp:`)
- JWT capability tokens (HMAC-SHA256, `iss/sub/exp/scope`)
- Interactive challenge lifecycle: create → approve/deny → consume
- Policy enforcement: `rateLimit`, `allowOrigins`, `allowAgents`
- HTML rendering via Tera template engine with progressive load strategy
- OpenAI-compatible tool JSON compilation with `x-w4` extension block
- Stable library API: `RuntimeOptions` / `build_default_runtime`

**Quick start:**

```bash
git clone https://github.com/web4-org/web4.git
cd web4
cargo run -p web4-gateway -- config.showcase.json
```

```bash
# Human view
open http://127.0.0.1:8090/

# Agent view — discover tools
curl -H "Accept: application/w4+json" http://127.0.0.1:8090/ | jq .services

# Invoke an open service
curl -X POST http://127.0.0.1:8090/services/math.open_local_add.w4s \
  -H "Content-Type: application/json" -d '{"a": 3, "b": 4}'
```

See **[docs/getting-started/quickstart.md](docs/getting-started/quickstart.md)** for the full
walkthrough, including all three consent modes.

---

## Documentation

| | |
|---|---|
| [Overview](docs/getting-started/overview.md) | What Web4 is and why |
| [Quickstart](docs/getting-started/quickstart.md) | Running the showcase |
| [W4ML Language](docs/concepts/w4ml-language.md) | Authoring W4 pages |
| [Consent Modes](docs/concepts/consent-modes.md) | open / capability / interactive |
| [Agent Integration](docs/guides/agent-integration.md) | Consuming W4 pages from agent code |
| [Architecture](docs/development/architecture.md) | Internal pipeline deep-dive |
| [Specification](WEB4ORG-STD-0001.md) | WEB4ORG-STD-0001 v0.1 (normative) |

---

## Status

Web4 is at a very early stage. The specification (v0.1) and this reference implementation are
functional — the core pipeline works end to end — but this is research infrastructure, not
production software. Breaking changes to the spec and APIs should be expected. Several planned
features (MCP server integration, persistent challenge storage, agent runtime conformance hooks)
are not yet implemented.

---

## Get Involved

Web4 is at a very early stage and the team is small. There is a lot left to build: MCP
integration, agent runtime hooks, persistent storage, more binding types, broader test
coverage, and the specification itself still has open questions.

If you share the vision and want to contribute — whether on the spec, the Rust implementation,
or the surrounding tooling — we would be glad to hear from you.

**[lpoems@icloud.com](mailto:lpoems@icloud.com)**

---

## License

Apache License 2.0
