# Devflow Verification Guide

This document outlines how to verify the Devflow project and its examples.

## Core Project Verification

To run the full local CI suite (formatting, linting, building, and testing) for the core project, use:

```bash
make check
```

This command uses `dwf` (the Devflow CLI) to orchestrate the checks across all crates in the workspace.

## Examples Verification

Devflow includes several examples to demonstrate multi-stack and containerized workflows.

### Automatic Verification

You can verify the primary examples (rust-lib and python-ext) using:

```bash
make verify-examples
```

### Manual Verification of Specific Examples

#### rust-lib (Rust, Containerized)

This example uses the `rust` stack and defaults to a containerized profile.

```bash
cd examples/rust-lib
dwf check:pr
```

#### python-ext (Subprocess Extension)

This example demonstrates the Devflow subprocess extension protocol.

```bash
cd examples/python-ext
dwf check:pr
```

*Note: This command requires an absolute path to the extension binary in `devflow.toml` if it's not on your PATH.*

#### node-ts (Node.js/TypeScript)

```bash
cd examples/node-ts
npm install
dwf check:pr
```

## Architecture Support

The `Dockerfile.devflow` templates in the examples are designed to be architecture-agnostic. They dynamically detect whether they are running on `x86_64` (standard CI/Linux) or `aarch64` (local Apple Silicon) to download the correct toolchain binaries.

## Troubleshooting

- **Docker/Podman**: Ensure a container engine is running if using `profile = "container"`. Devflow will automatically detect `podman` or `docker`.
- **Toolchains**: For "host" profiles, ensure the required language toolchains (Rust, Node, Python) are installed.
