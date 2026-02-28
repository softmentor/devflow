---
title: Dogfooding Strategy
label: devflow.developer-guide.dogfooding
---

# Dogfooding Strategy

Using Devflow to run Devflow itself is valuable, but it should be phased.

## Benefits

- validates real-world workflow fit continuously
- catches command and profile regressions early
- forces clear UX and documentation

## Risks Before Stable Release

- bootstrap fragility (tool must build before it can orchestrate)
- circular breakage (`dwf` regression can block all project checks)
- slower debugging if orchestration and product issues are mixed

## Recommended Approach

1. Keep direct `cargo` fallback commands documented and working.
2. Use `dwf` for non-critical local checks first.
3. Gate CI with `dwf` only after command and config contracts are stable.
4. Maintain a minimal emergency CI path (`cargo check/test`) during early releases.
