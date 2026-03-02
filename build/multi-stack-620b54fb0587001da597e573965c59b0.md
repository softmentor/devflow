---
title: Multi-Stack Projects
label: devflow.user-guide.multi-stack
---

# Multi-Stack Projects

Modern architectures often depend on multiple toolchains co-existing within the same repository (e.g., a **Tauri** desktop app needing both `node` for the frontend and `rust` for the backend, or a full-stack monorepo).

Devflow's container execution proxy seamlessly supports multiplexing extensions inside a single execution environment.

## Configuring `devflow.toml`

Instead of declaring a single stack, supply an array of requested stacks, and explicitly enable them in your `extensions` block:

```toml
[project]
name = "tauri-app"
stack = ["rust", "node"]

[runtime]
profile = "container"

[container]
image = "tauri-ci"
engine = "podman"

[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit"]

[extensions.rust]
source = "builtin"
required = true

[extensions.node]
source = "builtin"
required = true
```

## The Multi-Stack Container Environment

When generating your `Dockerfile.devflow`, you must build a unified image containing **all** requested toolchains. 

We recommend utilizing Multi-Stage Docker builds. Map standard `debian` dependencies in the base stage, and layer `rustc` and `node` in the final `ci` stage. This avoids rebuilding large LLVM toolchains when only simple `package.json` updates occur.

### Example Multi-Stack Dockerfile (`tauri`)
```dockerfile
FROM debian:12.8-slim AS base
# ... install curl, git, build-essential ...

FROM rust:1.93.1-slim-bookworm AS ci
COPY --from=base / /
# Install Node alongside Rust
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && apt-get install -y nodejs
```

> [!IMPORTANT]
> Never use moving tags like `:latest` or setup scripts pointing to moving URLs if strict caching is critical. Pinning explicitly to `debian:12.8-slim` and downloading exact architecture binaries (e.g., `sccache-v0.8.1-aarch64`) will ensure deterministic image fingerprinting.

## Execution Multiplexing

When Devflow parses `stack = ["rust", "node"]` under `profile = "container"`, it will dynamically iterate across **both** extensions:

1. Devflow queries the `node` extension and receives the `~/.npm` volume mapping.
2. Devflow queries the `rust` extension and receives the `.cargo` and `sccache` mapping.
3. Devflow launches *one* single proxy container merging all volumes:
   `podman run -v .cache/devflow/node/npm:/root/.npm -v .cache/devflow/rust/cargo:/usr/local/cargo ... tauri-ci`

This enables a command like `dwf check:pr` to seamlessly execute Node linters and Rust static analysis inside the exact same container state.
