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

    fn cache_mounts(&self) -> Vec<String> {
        vec!["node/npm:/root/.npm".to_string()]
    }

    fn fingerprint_inputs(&self) -> Vec<String> {
        vec![
            "package-lock.json".to_string(),
            "yarn.lock".to_string(),
            "pnpm-lock.yaml".to_string(),
            "package.json".to_string(),
        ]
    }
}

/// Helper for constructing `ExecutionAction`s concisely.
fn action(program: &str, args: &[&str]) -> ExecutionAction {
    ExecutionAction {
        program: program.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
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
        let ext = NodeExtension::new();
        assert_eq!(ext.name(), "node");
    }

    #[test]
    fn unit_test_capabilities_exist() {
        let ext = NodeExtension::new();
        let caps = ext.capabilities();
        assert!(caps.contains("build:debug"));
        assert!(caps.contains("setup"));
        assert!(caps.contains("lint:static"));
    }

    #[test]
    fn unit_test_valid_build_actions() {
        let ext = NodeExtension::new();

        let tests = vec![
            (cmd(PrimaryCommand::Setup, Some("deps")), "npm ci"),
            (cmd(PrimaryCommand::Lint, Some("static")), "npm run lint"),
            (cmd(PrimaryCommand::Test, Some("unit")), "npm run test:unit"),
            (
                cmd(PrimaryCommand::Package, Some("artifact")),
                "npm pack --dry-run",
            ),
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
        let ext = NodeExtension::new();

        let invalid_cmds = vec![
            cmd(PrimaryCommand::Fmt, Some("format")), // fmt:check and fmt:fix exist, not fmt:format
            cmd(PrimaryCommand::Setup, Some("toolchain")),
        ];

        for input_cmd in invalid_cmds {
            assert!(ext.build_action(&input_cmd).is_none());
        }
    }
}
