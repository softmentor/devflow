//! Devflow extension for Node.js projects.
//!
//! Provides the [`NodeExtension`] which maps Devflow [`CommandRef`]s into practical
//! `npm` commands, enabling JavaScript/TypeScript workflows to integrate transparently
//! into the Devflow ecosystem.

use devflow_core::{CommandRef, ExecutionAction, Extension};
use std::collections::HashSet;

/// The Devflow extension for Node.js.
///
/// Discovers project capabilities and maps primary Devflow actions into
/// localized `npm` invocations (e.g., `npm run build`, `npm ci`).
#[derive(Debug, Default)]
pub struct NodeExtension;

impl NodeExtension {
    /// Constructs a new [`NodeExtension`].
    pub fn new() -> Self {
        Self
    }
}

impl Extension for NodeExtension {
    fn name(&self) -> &str {
        "node"
    }

    fn capabilities(&self) -> HashSet<String> {
        [
            "setup",
            "fmt:check",
            "fmt:fix",
            "lint:static",
            "build:debug",
            "build:release",
            "test:unit",
            "test:integration",
            "package:artifact",
            "check",
            "release",
            "ci:generate",
            "ci:check",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect()
    }

    fn build_action(&self, cmd: &CommandRef) -> Option<ExecutionAction> {
        let primary = cmd.primary.as_str();
        let selector = cmd.selector.as_deref().unwrap_or("");

        match (primary, selector) {
            ("setup", "deps") => Some(action("npm", &["ci"])),
            ("setup", "doctor") => Some(action("npm", &["--version"])),
            ("fmt", "check") => Some(action("npm", &["run", "fmt:check"])),
            ("fmt", "fix") => Some(action("npm", &["run", "fmt:fix"])),
            ("lint", "static") => Some(action("npm", &["run", "lint"])),
            ("build", "debug") => Some(action("npm", &["run", "build"])),
            ("build", "release") => Some(action("npm", &["run", "build"])),
            ("test", "unit") => Some(action("npm", &["run", "test:unit"])),
            ("test", "integration") => Some(action("npm", &["run", "test:integration"])),
            ("test", "smoke") => Some(action("npm", &["run", "test:smoke"])),
            ("package", "artifact") => Some(action("npm", &["pack", "--dry-run"])),
            _ => None,
        }
    }
}

/// Helper for constructing `ExecutionAction`s concisely.
fn action(program: &str, args: &[&str]) -> ExecutionAction {
    ExecutionAction {
        program: program.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
    }
}
