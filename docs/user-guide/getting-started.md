---
title: Getting Started
label: devflow.user-guide.getting-started
---

# Getting Started

This quickstart is for first-time users who want Devflow running in a repository, regardless of implementation language.

## What You Will Achieve

By the end of this page you will:

- initialize a repository with `dwf init` (auto-detect or explicit template)
- run a local quality profile (`check:pr`)
- generate a CI workflow from config (`ci:generate`)
- validate CI workflow consistency (`ci:check`)

## Step 1: Install Devflow

Follow [Installation](#devflow.user-guide.installation).

## Step 2: Initialize Project Config and Starter CI

Recommended:

```bash
dwf init
```

This auto-detects project type in this order:

- `Cargo.toml` -> `rust`
- `tsconfig.json` -> `tsc`
- `package.json` -> `node`

Explicit template selection:

```bash
dwf init rust
dwf init node
dwf init tsc
dwf init kotlin
```

What this writes:

- `devflow.toml`
- `.github/workflows/ci.yml`

Use `--force` to overwrite existing files:

```bash
dwf --force init rust
```

If you prefer manual setup, create `devflow.toml` in your repository root.

TypeScript example:

```toml
[project]
name = "my-ts-repo"
stack = ["node"]

[runtime]
# Runtime execution profile: container | host | auto
profile = "auto"

[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration"]

[extensions.node]
source = "builtin"
required = true
```

## Step 3: Run Local Quality Gate

```bash
dwf check:pr
```

What this does:

- loads the `targets.pr` command list from `devflow.toml`
- runs each listed command locally, in order
- returns non-zero exit code if any command fails

What this does not do:

- it does not call remote CI services
- it does not push commits or modify your Git provider settings

## Step 4: Generate and Validate CI Workflow

```bash
dwf ci:generate
dwf ci:check
```

Expected behavior:

- `ci:generate` writes `.github/workflows/ci.yml`
- re-running `ci:generate` re-syncs the file from current config
- `ci:check` validates topology and detects drift between committed workflow and generated workflow

## Step 5: Inspect Planned Profiles

```bash
dwf ci:plan
```

This prints the target profile names currently defined in `[targets]`.
