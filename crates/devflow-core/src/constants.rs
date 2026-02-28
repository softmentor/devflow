//! Constants used across the Devflow workspace.

/// The filename for Devflow's primary configuration.
pub const CONFIG_FILE: &str = "devflow.toml";

/// The manifest file for Rust projects.
pub const MANIFEST_RUST: &str = "Cargo.toml";

/// The manifest file for Node and TypeScript projects.
pub const MANIFEST_NODE: &str = "package.json";
pub const MANIFEST_TSC: &str = "tsconfig.json";

/// Standard build system files for custom stacks.
pub const TARGET_CUSTOM_JUST: &str = "justfile";
pub const TARGET_CUSTOM_MAKE: &str = "Makefile";
