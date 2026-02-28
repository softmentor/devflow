---
title: Configuration
label: devflow.user-guide.configuration
---

# Configuration

`devflow.toml` is the source of truth for workflow behavior.

## Full Example

```toml
[project]
name = "my-project"
stack = ["rust", "node"]

[runtime]
# Runtime execution profile for command execution
# Allowed values: container | host | auto
profile = "auto"

[targets]
# Profile names are user-defined map keys.
# Common names: pr, main, release, staging, hotfix
pr = ["fmt:check", "lint:static", "build:debug", "test:unit", "test:integration"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration", "test:smoke"]
staging = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration", "package:artifact"]

[extensions.rust]
source = "builtin"
required = true
# Optional explicit capabilities. If omitted for builtin extension,
# default capabilities are loaded from the extension crate.
# capabilities = ["fmt:check", "lint:static", "test:unit"]

[extensions.node]
source = "builtin"
required = false

# Example path extension
[extensions.custom]
source = "path"
path = "./tools/devflow-ext-custom"
required = false
capabilities = ["lint:policy"]
```

## Section Details

### `[project]`

- `name`: logical project name used by generated outputs.
- `stack`: list of enabled stacks.
  - Allowed values today: `rust`, `node`, `custom`
  - `custom` delegates canonical commands to `justfile` or `Makefile` targets.

### `[runtime]`

- `profile`: runtime execution profile.
  - `container`: force container-oriented execution mode
  - `host`: run directly on host toolchain
  - `auto`: choose best available mode (default)

### `[targets]`

- dynamic profile map used by `check:<profile>` and CI generation.
- each profile value is an ordered list of canonical command selectors.
- examples: `check:pr`, `check:main`, `check:staging`

### `[extensions.<name>]`

- `source`: `builtin` or `path`
- `path`: required when `source = "path"`
- `required`: if true, load/validation failure is fatal
- `capabilities`: optional explicit capability list

## `custom` Stack Command Mapping

When `stack = ["custom"]`, Devflow maps selectors to `just`/`make` targets by replacing `:` with `-`.

Examples:

- `fmt:check` -> `fmt-check`
- `lint:static` -> `lint-static`
- `build:debug` -> `build-debug`
- `test:unit` -> `test-unit`

Execution order:

1. use `just <target>` when `justfile` exists and `just` is installed
2. otherwise use `make <target>` when `Makefile` exists

## Validation Rules

- unknown config keys fail
- invalid command syntax in target profiles fails
- unsupported selectors relative to loaded extensions fail
