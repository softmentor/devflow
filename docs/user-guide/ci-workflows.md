---
title: CI Workflows
label: devflow.user-guide.ci-workflows
---

# CI Workflows

## Goal

Keep CI workflow files synchronized with `devflow.toml` policy and command contracts.

## `ci:generate`

```bash
dwf ci:generate
```

What it does:

- generates workflow YAML based on current config (`targets.pr`)
- writes to `.github/workflows/ci.yml` by default
- overwrites existing file content with the latest generated contract

If you modify config locally and run `ci:generate` again, it re-syncs the workflow file.

Custom output path:

```bash
dwf --ci-output .github/workflows/devflow-ci.yml ci:generate
```

Preview without writing:

```bash
dwf --stdout ci:generate
```

## `ci:check`

```bash
dwf ci:check
```

What it checks:

- required workflow topology (`prep`, `build`, profile-derived `check_*` jobs)
- command coverage for `targets.pr`
- drift between on-disk workflow and expected generated output

If drift is detected, run:

```bash
dwf ci:generate
```

## `ci:plan`

```bash
dwf ci:plan
```

Shows configured target profile names (`pr`, `main`, `release`, custom profiles).

## Security Hardening (Least Privilege)

Devflow's generated CI workflows follow the Principle of Least Privilege. By default, the `GITHUB_TOKEN` is restricted to:

```yaml
permissions:
  contents: read
```

This ensures that CI jobs can checkout code and read repository metadata but cannot accidentally write back to the repository (e.g., creating tags, publishing releases, or modifying issues) unless explicitly granted per-job or per-step permissions.

## Execution Environment

### Shell Requirements
The generated workflow requires `/bin/bash` for advanced parallel execution tracking. This is handled automatically by the containerized environment (Debian-based), but ensures that process PIDs are tracked correctly during the `verify` phase.

### Parallel Execution
To optimize CI speed, Devflow executes checks in parallel within the same container. This is managed by a background process tracking script:

1. Starts checks in the background (`&`).
2. Collects PIDs (`pids+=($!)`).
3. Waits for all PIDs and accumulates exit codes.
4. Fails the job if any check fails.
