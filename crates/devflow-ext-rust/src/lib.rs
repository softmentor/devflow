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
            ("test", "unit") => Some(action("cargo", &["nextest", "run", "--lib", "--bins"])),
            ("test", "integration") => Some(action("cargo", &["test", "--tests"])),
            ("test", "smoke") => Some(action("cargo", &["test", "smoke"])),
            ("package", "artifact") => Some(action("cargo", &["build", "--release"])),
            ("release", "candidate") => Some(action("cargo", &["build", "--release"])),
            _ => None,
        }
    }

    fn cache_mounts(&self) -> Vec<String> {
        vec![
            "rust/cargo:/workspace/.cargo-cache".to_string(),
            "rust/target:/workspace/target/ci".to_string(),
        ]
    }

    fn env_vars(&self) -> std::collections::HashMap<String, String> {
        let mut env = std::collections::HashMap::new();
        env.insert(
            "CARGO_HOME".to_string(),
            "/workspace/.cargo-cache".to_string(),
        );
        env.insert(
            "CARGO_TARGET_DIR".to_string(),
            "/workspace/target/ci".to_string(),
        );
        env.insert(
            "SCCACHE_DIR".to_string(),
            "/workspace/.cargo-cache/sccache".to_string(),
        );
        env.insert("RUSTC_WRAPPER".to_string(), "sccache".to_string());
        env
    }

    fn fingerprint_inputs(&self) -> Vec<String> {
        vec![
            "Cargo.lock".to_string(),
            "rust-toolchain.toml".to_string(),
            "Cargo.toml".to_string(),
        ]
    }
}

/// Helper for constructing `ExecutionAction`s concisely.
fn action(program: &str, args: &[&str]) -> ExecutionAction {
    ExecutionAction {
        program: program.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        env: std::collections::HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devflow_core::PrimaryCommand;

    fn cmd(primary: PrimaryCommand, selector: Option<&str>) -> CommandRef {
        CommandRef {
            primary,
            selector: selector.map(|s| s.to_string()),
        }
    }

    #[test]
    fn smoke_test_extension_instantiation() {
        let ext = RustExtension::new();
        assert_eq!(ext.name(), "rust");
    }

    #[test]
    fn unit_test_capabilities_exist() {
        let ext = RustExtension::new();
        let caps = ext.capabilities();
        assert!(caps.contains("build:release"));
        assert!(caps.contains("test:smoke"));
        assert!(caps.contains("fmt:check"));
    }

    #[test]
    fn unit_test_valid_build_actions() {
        let ext = RustExtension::new();

        let tests = vec![
            (
                cmd(PrimaryCommand::Setup, Some("doctor")),
                "cargo --version",
            ),
            (
                cmd(PrimaryCommand::Build, Some("release")),
                "cargo build --release",
            ),
            (
                cmd(PrimaryCommand::Test, Some("integration")),
                "cargo test --tests",
            ),
            (cmd(PrimaryCommand::Fmt, Some("fix")), "cargo fmt --all"),
        ];

        for (input_cmd, expected_shell) in tests {
            let action = ext
                .build_action(&input_cmd)
                .expect("Expected valid action mapping");
            let actual_shell = format!("{} {}", action.program, action.args.join(" "));
            assert_eq!(actual_shell, expected_shell);
        }
    }

    #[test]
    fn unit_test_invalid_build_actions_return_none() {
        let ext = RustExtension::new();

        let invalid_cmds = vec![
            cmd(PrimaryCommand::Build, Some("unknown-target")),
            cmd(PrimaryCommand::Package, Some("docker")),
        ];

        for input_cmd in invalid_cmds {
            assert!(ext.build_action(&input_cmd).is_none());
        }
    }
}
