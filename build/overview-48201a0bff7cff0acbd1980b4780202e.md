---
title: Infrastructure Overview
label: devflow.developer-guide.infrastructure.overview
---

# Infrastructure Overview

Devflow currently supports GitHub Actions workflow generation via `devflow-gh`.

Target CI topology contract:

- `prep`
- `build`
- `check_*` jobs mapped from `targets.pr`
