---
title: Container and Cache Execution Design
label: devflow.architecture.container-execution
---

# Container and Cache Execution Design

This document covers the low-level interactions that enable Devflow to orchestrate fast, deterministic container execution while maintaining a strictly stack-agnostic core.

## 1. The Stack-Agnostic Boundary

A core invariant of Devflow's design is that **orchestration is generic; semantics are specific**.

*   **`devflow-core` (Agnostic)**: Manages state, parses TOML, and coordinates the DAG executor. It has no knowledge of language toolchains.
*   **`devflow-cli` (Agnostic)**: Reaches out to extensions to ask "What environment variables and volume mounts do you need?" It blindly translates `ExecutionAction`s (command + array of arguments) into `docker run ...` proxies.
*   **`devflow-ext-*` (Stack-Aware)**: Extensions own all the knowledge. 
    *   They return absolute volume mapping requirements (e.g., `devflow-ext-node` returns `~/.npm`).
    *   They define their hashable fingerprint inputs (e.g., `devflow-ext-rust` inputs `Cargo.lock` and `rust-toolchain.toml`).
    *   They define execution optimizations (e.g., mapping a `test` command internally to `cargo nextest run`).

Whether the runtime profile is `host` or `container`, the Extension behavior remains identical. If the profile is containerized, the `devflow-cli` executor seamlessly wraps the extension's generic Output Action into a container volume-mounted string without the extension knowing.

## 2. Container Lifecycle & Engine Determinism

By default, Devflow projects should utilize `profile = "container"` to guarantee environment reproducibility.

*   **Engine Determinism**: The container engine is explicit (`engine="docker"`, `"podman"`, or `"auto"`). If a project specifies `"docker"` and the developer does not have it installed on their `$PATH`, Devflow will **fail fast** rather than attempting an unsafe fallback.
*   **Fingerprint & Identity**: Devflow hashes the inputs specified by the active extensions (e.g., `Dockerfile`, `package.json`, `Cargo.lock`) into a unified SHA256 string. This fingerprint maps exactly to the CI `<image-tag>` hosted on `ghcr.io/org/repo-ci:hash`.
*   **CI Warm Reuse**: Starting containers is expensive. GitHub Actions CI pipelines enforce a `build` job that sequentially prepares the workspace and compiles dependencies inside the newly pulled container. **All subsequent parallel verify jobs** (`fmt`, `lint`, `test`) must re-mount this read-only warm cache directory array and boot up the identical image, entirely avoiding repeated compilation steps.

## 3. The Unified Cache (`DWF_CACHE_ROOT`)

Devflow implements an aggressive caching strategy pushing dependencies out of ephemeral containers onto the long-lived host disk.

*   **Mapping**: All operations map to the unified `DWF_CACHE_ROOT` (default `.cache/devflow`). If `devflow-ext-rust` asks to cache `.cargo/registry`, Devflow will generically volume mount `$DWF_CACHE_ROOT/cargo/registry:/root/.cargo/registry`. 
*   **Host vs Container**: These identical paths are utilized whether a user runs inside a container or locally on the host. This prevents massive file duplication on the developer's laptop.

## 4. Debugging and Troubleshooting

When container execution issues arise, developers need clear visibility into the proxy translation.

*   **Tracing the Proxy**: Prefix commands with `RUST_LOG=devflow=debug dwf verify`. Devflow will print the exact translated `docker run` execution string, making it easy to see if a volume mapping is malformed or an environment variable is dropped.
*   **Investigating the Container Environment**: Since Devflow hashes image identities, developers can `docker run -it --entrypoint /bin/sh ghcr.io/org/repo-ci:<fingerprint>` to directly inspect the execution environment that Devflow is using.
*   **Dropping into CI Shells**: Since the local and CI environments use the *identical* docker image, developers can replicate CI failures completely locally by forcing `profile=container`.

## 5. Teardown and Cache Invalidations

Developers risk exhausting their local disk space if abandoned container images and `$DWF_CACHE_ROOT` binaries accumulate unbounded over time.

*   **Image Pruning**: The CLI executor leverages `--rm` on every `docker run` initialization to prevent stopped container leakage. However, large base images persist.
*   **Cache Invalidation**: Devflow depends on toolchain-specific mechanisms inside the extensions to dictate cache pruning. (e.g., `sccache` manages its own LRU limits natively).
*   **`dwf cache prune` (Roadmap)**: Devflow will eventually expose a top-level cache sub-command which sweeps the `DWF_CACHE_ROOT` and prunes directories older than a configured timestamp boundary, alongside dropping dangling docker/podman daemon images mapped to old, abandoned fingerprint hashes.
