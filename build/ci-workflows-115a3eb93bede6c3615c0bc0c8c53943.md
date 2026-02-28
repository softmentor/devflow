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
