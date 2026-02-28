# Using Custom Extensions

Devflow separates its core orchestration engine from the logic used to build, test, and package your software. The logic is handled by **Extensions**—specialized programs that connect to Devflow.

While Devflow comes with official extensions for languages like Rust or Node.js, you may need to use **Custom Extensions** provided by your organization or the open-source community. Let's look at how to use and manage extensions.

## How Extensions Work

When you type `devflow test`, Devflow doesn't actually know how to run tests for your project. Instead, it:
1. Searches your system for installed extensions.
2. Identifies which extension claimed support for the `test` capability.
3. Delegates the heavy lifting to that extension, which then figures out whether to call `cargo test`, `npm run test`, `pytest`, etc.

## Finding and Installing Extensions

Extensions are simply executable binaries or scripts in your `$PATH` that start with the `devflow-ext-` prefix. 
To install an extension:
- **Download the Binary**: Drop the `devflow-ext-xyz` binary into a folder that falls into your system `$PATH` (e.g., `/usr/local/bin` or your `~/bin` path).
- **Verify it's Executable**: Make sure your operating system is permitted to run it (`chmod +x devflow-ext-xyz` on Linux/macOS).

Once in your `$PATH`, Devflow automatically discovers it the next time you run a command.

## Troubleshooting Extensions

If a command fails because an extension isn't running as expected, you can manually isolate the problem without running Devflow.

1. **Check if Devflow sees the extension:**
   Execute the extension directly with the `--discover` flag to ensure it's responding with capabilities.
   ```bash
   # Make sure the extension is in your PATH.
   devflow-ext-myplugin --discover
   ```
   *Expected output: `["test", "build", "fmt"]`*
   
2. **Diagnose capability gaps:**
   If you get an error saying Devflow does not expose a capability like `test:integration`, you can check the JSON output of the discover command above to confirm whether the author missed it.
   
3. **Check execution actions:**
   You can mock what Devflow does by streaming a JSON payload to the extension:
   ```bash
   echo '{"primary": "test"}' | devflow-ext-myplugin --build-action
   ```
   You should see a printed JSON object showing what program the extension is trying to invoke (e.g. `{"program": "pytest", "args": ["tests/"]}`).

## Developing Your Own

Devflow extensions can be built in **any language**—Go, Python, Bash, Rust, etc.—because they communicate via simple standard JSON payload I/O. For detailed tutorials on creating your own custom extensions, please check our [Developer Guide](../developer-guide/developer-index.md).
