---
title: Installation
label: devflow.user-guide.installation
---

# Installation

## Prerequisites

- macOS or Linux shell environment
- Rust toolchain (required only for current source-based install path)
- `cargo` available in PATH

You do not need to know Rust internals to use Devflow, but the current bootstrap build path uses Rust until binary distribution channels are published.

## Install via Script

From repository root:

```bash
./install.sh
```

What this does:

- builds `dwf` in release mode
- installs binary to `${HOME}/.local/bin/dwf` (or custom `DWF_INSTALL_DIR`)

## Verify Installation

```bash
dwf --help
dwf ci:plan
```

If `dwf` is not found, add `${HOME}/.local/bin` to your PATH.
