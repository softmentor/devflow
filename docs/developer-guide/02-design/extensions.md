---
title: Extension Model
label: devflow.developer-guide.design.extensions
---

# Extension Model

Extension entries are declared in `devflow.toml` under `[extensions.*]`.

Supported sources:

- `builtin`
- `path`

Validation includes:

- API version checks
- path existence (for `source = "path"`)
- capability coverage for target profiles
