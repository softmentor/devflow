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

## Primary Commands

| Command | Purpose | Typical Usage | Mutates Files? |
| --- | --- | --- | --- |
| `init` | bootstrap config and starter CI workflow | `init`, `init rust`, `init kotlin` | yes |
| `setup` | prepare dependencies and environment checks | `setup:doctor`, `setup:deps` | sometimes |
| `fmt` | formatting checks/fixes | `fmt:check`, `fmt:fix` | `fmt:fix` yes |
| `lint` | static/policy analysis | `lint:static` | no |
| `build` | compile/package prep builds | `build:debug`, `build:release` | build artifacts only |
| `test` | execute test scopes | `test:unit`, `test:integration`, `test:smoke` | no |
| `package` | prepare distributable outputs | `package:artifact` | artifacts only |
| `check` | run a configured quality profile | `check:pr`, `check:main` | depends on profile |
| `release` | release-oriented tasks | `release:candidate` | artifacts/metadata |
| `ci` | CI workflow generation and validation | `ci:generate`, `ci:check`, `ci:plan` | `ci:generate` yes |

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
