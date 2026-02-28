---
title: Troubleshooting
label: devflow.user-guide.user-troubleshooting
---

# Troubleshooting

## Unknown command selector in targets

Run `dwf check:pr` and fix unsupported selectors in `targets.*`.

## Extension path errors

For `source = "path"`, ensure `path` exists and is readable.

## Tool not found during execution

Install required toolchain commands (`cargo`, `npm`, etc.) for configured stacks.
