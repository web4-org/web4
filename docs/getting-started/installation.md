# Installation

## Prerequisites

| Requirement | Minimum | Notes |
|---|---|---|
| **Rust** | 1.75 | Install via [rustup](https://rustup.rs) |
| **Cargo** | bundled with Rust | Workspace resolver v2 required |
| **Python 3** | 3.8+ | Only needed to run the local-binding examples |
| **Git** | any | For cloning |

---

## Clone the Repository

```bash
git clone https://github.com/your-org/web4.git
cd web4
```

---

## Build the Workspace

Build all crates (library + gateway binary):

```bash
cargo build --workspace
```

For a release build:

```bash
cargo build --workspace --release
```

The gateway binary is placed at:

```
target/debug/web4-gateway       # debug build
target/release/web4-gateway     # release build
```

---

## Verify the Build

Run the workspace test suite to confirm everything is working:

```bash
cargo test --workspace
```

Expected output ends with something like:

```
test result: ok. N passed; 0 failed; ...
```

Run a quick compile-check without producing binaries:

```bash
cargo check --workspace
```

---

## Lint and Format

Before contributing, always run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Running the Gateway

The gateway requires a JSON config file. A ready-made example is provided:

```bash
cargo run -p web4-gateway -- config.showcase.json
```

You should see:

```
INFO web4_gateway: web4 gateway listening addr=127.0.0.1:8090
```

See [Quickstart](quickstart.md) for what to do next, or [Configuring the Gateway](../guides/configuring-gateway.md) for all config options.
