use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

use tracing::{debug, error};

use crate::command::CommandRef;
use crate::extension::{ExecutionAction, Extension};

/// An extension that delegates to an external binary via JSON over stdio.
#[derive(Debug)]
pub struct SubprocessExtension {
    name: String,
    binary_path: String,
    capabilities: HashSet<String>,
}

impl SubprocessExtension {
    /// Creates a new `SubprocessExtension`.
    pub fn new(name: String, binary_path: String, capabilities: HashSet<String>) -> Self {
        Self {
            name,
            binary_path,
            capabilities,
        }
    }
}

impl Extension for SubprocessExtension {
    fn name(&self) -> &str {
        &self.name
    }

    fn capabilities(&self) -> HashSet<String> {
        self.capabilities.clone()
    }

    fn build_action(&self, cmd: &CommandRef) -> Option<ExecutionAction> {
        let serialized_cmd = match serde_json::to_string(cmd) {
            Ok(json) => json,
            Err(e) => {
                error!("failed to serialize command for {}: {}", self.name, e);
                return None;
            }
        };

        let mut child = match Command::new(&self.binary_path)
            .arg("--build-action")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                error!(
                    "failed to spawn extension binary '{}': {}",
                    self.binary_path, e
                );
                return None;
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(serialized_cmd.as_bytes()) {
                error!("failed to write to extension stdin: {}", e);
                return None;
            }
        }

        let output = match child.wait_with_output() {
            Ok(out) => out,
            Err(e) => {
                error!("failed to read from extension stdout: {}", e);
                return None;
            }
        };

        if !output.status.success() {
            debug!(
                "extension {} declined to build action for {}",
                self.name,
                cmd.canonical()
            );
            return None;
        }

        match serde_json::from_slice::<ExecutionAction>(&output.stdout) {
            Ok(action) => Some(action),
            Err(e) => {
                error!("failed to parse ExecutionAction from {}: {}", self.name, e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::PrimaryCommand;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn create_mock_extension(dir: &TempDir) -> String {
        let script_path = dir.path().join("mock-ext.py");
        let script_content = r#"#!/usr/bin/env python3
import sys
import json

if "--build-action" in sys.argv:
    input_data = sys.stdin.read()
    cmd = json.loads(input_data)
    if cmd.get("primary") == "test":
        print(json.dumps({"program": "echo", "args": ["mock-test"]}))
        sys.exit(0)
    else:
        sys.exit(1)
"#;
        fs::write(&script_path, script_content).unwrap();

        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();

        script_path.to_string_lossy().to_string()
    }

    #[test]
    fn subprocess_extension_build_action_success() {
        let dir = tempfile::tempdir().unwrap();
        let binary_path = create_mock_extension(&dir);

        let ext = SubprocessExtension::new(
            "mock".to_string(),
            binary_path,
            HashSet::from(["test".to_string()]),
        );

        let cmd = CommandRef {
            primary: PrimaryCommand::Test,
            selector: None,
        };

        let action = ext.build_action(&cmd).expect("should return action");
        assert_eq!(action.program, "echo");
        assert_eq!(action.args, vec!["mock-test".to_string()]);
    }

    #[test]
    fn subprocess_extension_build_action_failure() {
        let dir = tempfile::tempdir().unwrap();
        let binary_path = create_mock_extension(&dir);

        let ext = SubprocessExtension::new(
            "mock".to_string(),
            binary_path,
            HashSet::from(["test".to_string()]),
        );

        // Our python script exits with 1 for non-test commands
        let cmd = CommandRef {
            primary: PrimaryCommand::Build,
            selector: None,
        };

        let action = ext.build_action(&cmd);
        assert!(action.is_none());
    }
}
