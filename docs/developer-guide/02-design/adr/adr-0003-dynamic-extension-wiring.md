---
title: ADR-0003 Dynamic Extension Wiring and Subprocess Execution
label: devflow.developer-guide.design.adr-0003
date: 2026-02-27
status: accepted
---

# ADR-0003: Dynamic Extension Wiring and Subprocess Execution

## Context

Devflow needs to support multiple language stacks (`rust`, `node`, `python`, etc.) without bloating the core CLI binary with language-specific compilation or logic.
Initially, the `devflow-core` library directly depended on extension crates using hardcoded dispatch tables, which violated the stack-agnostic boundary of the core engine and prevented external developers from writing custom project extensions.

## Decision

We have decided to formalize an `Extension` trait and introduce dynamic runtime discovery.

1. **Extension Trait**: All extensions implement an `Extension` trait offering `name()`, `capabilities()`, and `build_action()`. `devflow-core` completely drops dependencies on specific stack implementations (`devflow-ext-rust`, `devflow-ext-node`).
2. **Dynamic Boot Wiring**: At boot, `devflow-cli` dynamically instantiates extensions and places them into an `ExtensionRegistry`.
3. **Subprocess Delegation**: For extensions not built into the CLI, Devflow will discover binaries in `$PATH` prefixed with `devflow-ext-`. It will probe these binaries by executing them with `--discover`, which returns a JSON array of supported capabilities.
4. **Execution Protocol**: To execute a targeted command, the Core serialization engine pipes a standard JSON `CommandRef` to the external binary via standard IO `stdin` (using `--build-action`). The binary returns an `ExecutionAction` (program and arguments) as JSON over `stdout`.

## Consequences

### Positive
- `devflow-core` is purely an orchestration engine, completely uncoupled from language specifics.
- Teams can write new Devflow extensions in *any language* (Go, Python, Bash) simply by writing a binary that responds to `--discover` and `--build-action` with JSON.
- Command mapping logic natively resides inside the specific technology stack crate instead of sprawling match statements inside `devflow-cli/src/executor.rs`.

### Negative
- Serialization overhead when passing Command contexts to sub-processes.
- Reliance on `$PATH` configurations means execution environments must be properly curated for subprocess discovery to succeed.
