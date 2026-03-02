---
title: Commands
label: devflow.user-guide.commands
---

# Commands

This page explains what each command family is for before listing concrete selectors.

## How to Read Command Names

- Primary command: high-level intent (`test`, `check`, `ci`)
- Selector: specific behavior (`test:unit`, `check:pr`, `ci:generate`)

General form:

```text
dwf <primary>:<selector>
```

## Command Glossary

Devflow commands are categorized by their role in the developer lifecycle.

### Initialization & Environment
| Command | Description |
| --- | --- |
| `init` | Bootstrap a project (detects stack automatically) |
| `setup:doctor` | Verify host toolchains and environment |
| `setup:deps` | Fetch and pre-cache project dependencies |
| `setup:toolchain` | Install/update required language toolchains |

### Verification & Security
| Command | Description |
| --- | --- |
| `check:pr` | Run the standard PR verification (fmt, lint, build, test) |
| `check:security` | Run local vulnerability scan on CI images (requires Trivy) |
| `test:unit` | Run project unit tests |
| `test:integration` | Run integration/infrastructure tests |
| `test:smoke` | Run high-level end-to-end smoke tests |

### Development Workflow
| Command | Description |
| --- | --- |
| `fmt:check` | Check if code matches project formatting standards |
| `fmt:fix` | Automatically apply formatting fixes |
| `lint:static` | Run clippy, eslint, or other static analyzers |
| `build:debug` | Perform an incremental debug build |
| `build:release` | Perform an optimized production build |

### CI Infrastructure
| Command | Description |
| --- | --- |
| `ci:generate` | Sync `.github/workflows/ci.yml` with `devflow.toml` |
| `ci:check` | Verify if local CI workflow matches current config |
| `ci:plan` | Preview the CI execution strategy and profiles |

### Maintenance & Release
| Command | Description |
| --- | --- |
| `prune:cache` | Cleanup local or GHA caches (use `--local`, `--gh`, `--all`) |
| `prune:runs` | Clean up stale GHA workflow runs (use `--gh`) |
| `package:artifact` | Build and bundle project distribution artifacts |
| `release:candidate` | Tag and prepare a new release candidate |

## Common Selectors

### Initialization

- `init`: auto-detect template from repository files
- `init:rust` or `init rust`: rust-oriented config and CI starter
- `init:node` or `init node`: node-oriented config and CI starter
- `init:tsc` or `init tsc`: typescript-oriented config and CI starter
- `init:kotlin` or `init kotlin`: custom stack config for Make/Just-backed Kotlin workflows

### Quality Profiles

- `check:pr`: runs `[targets].pr`
- `check:main`: runs `[targets].main`
- `check:<custom>`: runs any custom profile key defined under `[targets]`

### CI Lifecycle

- `ci:generate`: generate `.github/workflows/ci.yml` from config
- `ci:check`: validate on-disk workflow topology and detect drift
- `ci:plan`: list configured profile keys used by CI policy

### `fmt:check` vs `fmt:fix`

| Command | Responsibility | Typical Use |
| --- | --- | --- |
| `fmt:check` | verify formatting compliance only | CI and pre-merge checks |
| `fmt:fix` | apply formatter changes to files | local editing loop |

### Typical Local Loop

```bash
dwf fmt:check
dwf lint:static
dwf test:unit
dwf check:pr
```
