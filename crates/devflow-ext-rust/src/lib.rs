//! Devflow extension for Rust projects.
//!
//! Provides the [`RustExtension`] which maps Devflow [`CommandRef`]s into practical
//! `cargo` commands, enabling Rust workflows to integrate transparently
//! into the Devflow ecosystem.

use devflow_core::{CommandRef, ExecutionAction, Extension};
use std::collections::HashSet;

/// The Devflow extension for Rust.
///
/// Discovers project capabilities and maps primary Devflow actions into
/// localized `cargo` invocations (e.g., `cargo build`, `cargo clippy`).
#[derive(Debug, Default)]
pub struct RustExtension;

impl RustExtension {
    /// Constructs a new [`RustExtension`].
    pub fn new() -> Self {
        Self
    }
}

impl Extension for RustExtension {
    fn name(&self) -> &str {
        "rust"
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
            "test:smoke",
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
            ("setup", "toolchain") => Some(action("rustup", &["show"])),
            ("setup", "deps") => Some(action("cargo", &["fetch"])),
            ("setup", "doctor") => Some(action("cargo", &["--version"])),
            ("fmt", "check") => Some(action("cargo", &["fmt", "--all", "--", "--check"])),
            ("fmt", "fix") => Some(action("cargo", &["fmt", "--all"])),
            ("lint", "static") => Some(action(
                "cargo",
                &[
                    "clippy",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
            )),
            ("build", "debug") => Some(action("cargo", &["build"])),
            ("build", "release") => Some(action("cargo", &["build", "--release"])),
            ("test", "unit") => Some(action("cargo", &["test", "--lib", "--bins"])),
            ("test", "integration") => Some(action("cargo", &["test", "--tests"])),
            ("test", "smoke") => Some(action("cargo", &["test", "smoke"])),
            ("package", "artifact") => Some(action("cargo", &["build", "--release"])),
            ("release", "candidate") => Some(action("cargo", &["build", "--release"])),
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
