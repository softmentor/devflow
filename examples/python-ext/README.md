# Devflow Python Extension Example

This directory contains a sample Python implementation of a Devflow Phase 2 Extension using the JSON-RPC protocol over standard I/O streams.

Devflow has a stack-agnostic core which relies on dynamically discovering extensions in your `$PATH` prefixed with `devflow-ext-`.

## How it works

The `devflow-ext-python` script is a standalone, executable Python file that implements the two primary Devflow extension protocols:

### 1. Discovery Phase (`--discover`)

When Devflow starts or re-indexes its capabilities, it runs discovered binaries via:
```bash
devflow-ext-python --discover
```
The extension must print a JSON array of supported capabilities (like `"test"`, `"fmt"`, etc.) to its `stdout` and exit with code 0.

### 2. Action Building Phase (`--build-action`)

When Devflow decides to execute a requested command, it streams a JSON payload (`CommandRef`) to the extension binary via `stdin` and reads a serialized response (`ExecutionAction`) from `stdout`.

```bash
echo '{"primary": "test"}' | devflow-ext-python --build-action
```

This tells the extension "The user requested `test`". The extension replies with the exact command to execute. For example:
```json
{"program": "pytest", "args": ["tests/"]}
```

## Testing Locally

Grant execution permission to the script:
```bash
chmod +x devflow-ext-python
```

Run discovery:
```bash
./devflow-ext-python --discover
```

Run action building:
```bash
echo '{"primary": "test"}' | ./devflow-ext-python --build-action
echo '{"primary": "test", "selector": "lint"}' | ./devflow-ext-python --build-action
echo '{"primary": "fmt"}' | ./devflow-ext-python --build-action
```

## Wiring it into Devflow

Add this directory to your `$PATH` so the `devflow-cli` can discover the binary:
```bash
export PATH="$PWD:$PATH"
```
When running `devflow test`, Devflow will delegate to this python extension instead of its built-in match statements.
