---
title: New Stacks (Kotlin Example)
label: devflow.user-guide.new-stacks
---

# New Stacks (Kotlin Example)

If your stack is not natively mapped yet, use Devflow with `stack = ["custom"]`.

This keeps canonical Devflow commands while delegating implementation details to your existing build tool.

## Step 1: Initialize with Kotlin Template

```bash
dwf init kotlin
```

This generates a `custom` stack config and starter CI workflow.

## Step 2: Provide Targets in `justfile` or `Makefile`

Example `Makefile`:

```make
fmt-check:
	./gradlew ktlintCheck

lint-static:
	./gradlew detekt

build-debug:
	./gradlew build -x test

test-unit:
	./gradlew test

test-integration:
	./gradlew integrationTest
```

## Step 3: Run Canonical Commands

```bash
dwf check:pr
dwf ci:generate
dwf ci:check
```

## Roadblock Avoidance Checklist

- keep target names aligned with canonical selectors (`:` -> `-`)
- ensure your local runner (`just` or `make`) is installed in both local and CI images
- start with `fmt:check`, `lint:static`, `build:debug`, `test:unit`; add more selectors incrementally
- use `dwf --stdout ci:generate` to inspect workflow output before overwriting files
