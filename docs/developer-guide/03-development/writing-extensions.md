# Writing Custom Extensions

Devflow allows you to write custom workflow extensions in **any language**. Because Devflow interacts with extensions using a simple JSON over standard I/O (often referred to as a "JSON-RPC" protocol via `--discover` and `--build-action`), you are not constrained to writing Rust code to extend the CLI.

This guide explains how external developers can write, debug, and test Devflow extensions locally.

## The Extension Protocol

To create a Devflow extension, you need an executable script or binary whose name starts with a `devflow-ext-` prefix. Devflow uses this prefix to discover your logic.

Your extension must handle two CLI flags:
1. `--discover`: Announce what Devflow features you support.
2. `--build-action`: Receive a parsed command and tell Devflow what to run.

### Step 1: Handling `--discover`

When `devflow` runs or boots up, it looks for extension binaries in your `$PATH` and queries them using:
```bash
devflow-ext-myext --discover
```
Your script must output a JSON array of supported capabilities to `stdout` and exit with code `0`.

**Example Python implementation:**
```python
import sys
import json

if "--discover" in sys.argv:
    # Say that we support "test" and "fmt"
    print(json.dumps(["test", "fmt", "test:lint"]))
    sys.exit(0)
```

### Step 2: Handling `--build-action`

When a user runs a supported command (e.g., `devflow test`), Devflow matches the `"test"` capability to your extension. It then executes your extension, streaming a JSON `CommandRef` payload via `stdin`.

```bash
echo '{"primary": "test"}' | devflow-ext-myext --build-action
```

Your script reads this JSON struct, decides what underlying programs to call, and returns a JSON `ExecutionAction` to `stdout`.

**Example Python implementation:**
```python
if "--build-action" in sys.argv:
    # Read CommandRef from standard input
    input_data = sys.stdin.read()
    cmd_ref = json.loads(input_data)
    
    # Analyze the command
    primary = cmd_ref.get("primary")
    selector = cmd_ref.get("selector")
    
    # Tell Devflow what to execute
    if primary == "test":
        if selector == "lint":
            action = {"program": "flake8", "args": ["."]}
        else:
            action = {"program": "pytest", "args": ["tests/"]}
        
        # Print the execution action for devflow to run
        print(json.dumps(action))
        sys.exit(0)
```

## Developing and Debugging Locally

1. **Write your script**: Write a script (e.g. `devflow-ext-myext.sh` or `devflow-ext-python`) handling `--discover` and `--build-action`. Make sure it's executable (`chmod +x`).
2. **Add to PATH**: Add the directory containing your script to your system `$PATH` so Devflow can find it during discovery.
   ```bash
   export PATH="/path/to/your/script/dir:$PATH"
   ```
3. **Debug independent of Devflow**: Because the protocol is simply JSON over standard streams, you can test it directly in your terminal without Devflow:
   ```bash
   # Test discovery
   ./devflow-ext-python --discover
   
   # Test building an action
   echo '{"primary": "test", "selector": "lint"}' | ./devflow-ext-python --build-action
   ```
   *Expected manual test output:*
   `{"program": "flake8", "args": ["."]}`
4. **Integration Test with Devflow**: Once the CLI commands output correct JSON, you can run Devflow on any project and it will immediately delegate execution to your binary!

## Examples

We provide sample implementations of this protocol to guide your development:
- [Python Example Extension](https://github.com/organization/devflow/tree/main/examples/python-ext)
