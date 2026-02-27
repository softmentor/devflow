# devflow

`devflow` (binary: `dwf`) is a workflow orchestration CLI with a stable primary command surface and selector-based secondary commands.

## Workspace

- `crates/devflow-core`: command model, config, runtime profile, extension discovery.
- `crates/devflow-cli`: `dwf` binary entrypoint.
- `crates/devflow-policy`: policy expansion for `check:*` commands.
- `crates/devflow-gh`: CI workflow rendering stub.
- `crates/devflow-ext-rust`: Rust extension capability baseline.
- `crates/devflow-ext-node`: Node extension capability baseline.

## Quick Start

```bash
cargo run -p devflow-cli -- init rust
cargo run -p devflow-cli -- check:pr
cargo run -p devflow-cli -- test:unit
cargo run -p devflow-cli -- ci:generate
```

`ci:generate` writes to `.github/workflows/ci.yml` by default.
Use `--stdout` to print instead.

`devflow` complements existing `Makefile`/`justfile` workflows by standardizing command intent (`fmt:check`, `test:unit`, `check:pr`) and policy while still delegating stack-specific implementation details.

## Command Model

Primary commands:

- `init`
- `setup`
- `fmt`
- `lint`
- `build`
- `test`
- `package`
- `check`
- `release`
- `ci`

Selectors:

- `fmt:check`, `fmt:fix`
- `test:unit`, `test:integration`, `test:smoke`
- `check:pr`, `check:main`, `check:release`
- `ci:generate`, `ci:check`, `ci:plan`
