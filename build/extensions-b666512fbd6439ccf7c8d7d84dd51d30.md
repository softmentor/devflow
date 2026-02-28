---
title: Extension Model
label: devflow.developer-guide.design.extensions
---

# Extension Model

Extension entries are declared in `devflow.toml` under `[extensions.*]`.

Supported sources:

- `builtin`: Extensions directly compiled and linked into the CLI.
- `path`: (Planned) Local extensions loaded via path.
- `subprocess`: Binaries named `devflow-ext-<stack>` available in system `$PATH` that implement the standard JSON over `stdio` extension protocol.

Validation includes:

- API version checks
- capability coverage for target profiles
- JSON decode payload mapping for subprocesses
