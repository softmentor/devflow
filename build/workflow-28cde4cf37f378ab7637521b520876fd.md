---
title: Development Workflow
label: devflow.developer-guide.workflow
---

# Development Workflow

Recommended local sequence:

```bash
cargo fmt
cargo check --offline
cargo test --offline
cargo run -p devflow-cli -- check:pr
```
