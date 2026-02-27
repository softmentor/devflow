---
title: Quickstart
label: devflow.developer-guide.quickstart
---

# Quickstart

This section is for contributors who want to run and modify Devflow source code.
For now, use direct Cargo commands as the primary development path (dogfooding is phased later).

## Prerequisites

- Git
- Rust toolchain (`cargo`, `rustc`)

## Step-by-Step

1. Clone repository:

```bash
git clone https://github.com/softmentor/devflow.git
cd devflow
```

2. Build and validate:

```bash
cargo fmt
cargo check --offline
cargo test --offline
```

3. Run core workflow commands from source:

```bash
cargo run -p devflow-cli -- check:pr
cargo run -p devflow-cli -- ci:generate
cargo run -p devflow-cli -- ci:check
```

## Crate Overview

- `devflow-core`: canonical command model, config parsing/validation, extension registry.
- `devflow-cli`: executable entrypoint (`dwf`) and runtime command dispatch.
- `devflow-policy`: profile expansion for `check:<profile>`.
- `devflow-gh`: CI workflow generation and validation logic.
- `devflow-ext-rust`: builtin Rust capability declarations.
- `devflow-ext-node`: builtin Node capability declarations.
