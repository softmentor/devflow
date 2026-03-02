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
| Command | Description | Flags |
| --- | --- | --- |
| `prune:cache` | Cleanup local or GHA caches | `--local`, `--gh`, `--all`, `--force` |
| `prune:runs` | Clean up stale GHA workflow runs | `--gh`, `--all` |
| `package:artifact` | Build and bundle project distribution artifacts | |
| `release:candidate` | Tag and prepare a new release candidate | |

#### `make gh-setup` - GitHub Administration

Devflow provides Infrastructure-as-Code (Terraform) for managing GitHub repository settings.

- **Action:** Initializes Terraform in `.github/settings/terraform` and generates a plan for repository settings, branch protection, and security policies.
- **Requirement:** Requires `terraform` (automatically installed via `make setup-tools` on macOS).
- **Files Managed:**
  - Repository metadata (description, features)
  - Branch protection rules for `main`
  - Required status checks (matching `ci.yml`)

#### `Community Standards`

The project includes standard GitHub community files in the root and `.github` directory:
- **Found in `.github/ISSUE_TEMPLATE/`**: Structured templates for Bugs, Features, and Security Incidents.
- **Found in root**: `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md`.
- **Found in `.github/`**: `SECURITY.md`, `CODEOWNERS`, and `dependabot.yml`.

#### `make teardown` - Environment Reset

While `dwf` handles logic-level caches, the root `Makefile` provides a `teardown` target for a "scorched earth" local reset.

- **Action:**
  - Deletes `.cargo-cache`, `target/ci`, and `ci-image.tar`.
  - **Container Pruning:** Automatically detects `podman` or `docker` and runs `system prune -f` and `volume prune -f`.
- **Use Case:** Use this when you need to completely refresh the container engine state or clear up major disk space occupied by untagged images.

#### `prune:cache` - Deep Dive

This command is used to reclaim disk space or reset CI state. It supports granular target selection via flags.

**Local Pruning (`--local` or `--all`):**
- **Directories pruned:**
    - The directory set in `cache.root` in `devflow.toml` (defaults to `.cargo-cache`).
    - `target/ci`: The staging directory for CI-localized builds and images.
- **Reporting:** Displays total MB reclaimed.

**GitHub Actions Pruning (`--gh` or `--all`):**
- **Standard logic:**
    - Removes PR caches (`refs/pull/*`) older than 24 hours.
    - If total GH storage exceeds 8GB, it performs "LIFO" pruning on cargo caches (keeps the latest for each ref).
- **Force logic (`--force`):** Purges **all** caches for the repository immediately.
- **Requirement:** Requires the `gh` CLI to be installed and authenticated.

#### `prune:runs` - Deep Dive

Cleans up the GitHub Actions execution history. Requires `--gh` or `--all`.

- **Action:**
    - Automatically deletes all **Failed** and **Cancelled** runs.
    - Retains the **100 most recent** successful/completed runs, deleting everything older.
- **Requirement:** Requires the `gh` CLI.

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
