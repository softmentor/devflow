use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;
use tracing::{debug, error};

use crate::command::CommandRef;
use crate::extension::{ExecutionAction, Extension};

/// An extension that delegates to an external binary via JSON over stdio.
#[derive(Debug)]
pub struct SubprocessExtension {
    name: String,
    binary_path: String,
    capabilities: HashSet<String>,
    is_trusted: bool,
}

impl SubprocessExtension {
    /// Creates a new `SubprocessExtension`.
    pub fn new(
        name: String,
        binary_path: String,
        capabilities: HashSet<String>,
        is_trusted: bool,
    ) -> Self {
        Self {
            name,
            binary_path,
            capabilities,
            is_trusted,
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

    fn build_action(&self, cmd: &CommandRef) -> Result<Option<ExecutionAction>> {
        let serialized_cmd = serde_json::to_string(cmd)
            .map_err(|e| anyhow::anyhow!("failed to serialize command for {}: {}", self.name, e))?;

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
                return Ok(None);
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(serialized_cmd.as_bytes()) {
                error!("failed to write to extension stdin: {}", e);
                return Ok(None);
            }
        }

        let output = match child.wait_with_output() {
            Ok(out) => out,
            Err(e) => {
                error!("failed to read from extension stdout: {}", e);
                return Ok(None);
            }
        };

        if !output.status.success() {
            debug!(
                "extension {} declined to build action for {}",
                self.name,
                cmd.canonical()
            );
            return Ok(None);
        }

        let action = serde_json::from_slice::<ExecutionAction>(&output.stdout).map_err(|e| {
            anyhow::anyhow!("failed to parse ExecutionAction from {}: {}", self.name, e)
        })?;

        Ok(Some(action))
    }

    fn is_trusted(&self) -> bool {
        self.is_trusted
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
            true,
        );

        let cmd = CommandRef {
            primary: PrimaryCommand::Test,
            selector: None,
        };

        let action = ext
            .build_action(&cmd)
            .expect("RPC failed")
            .expect("should return action");
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
            true,
        );

        // Our python script exits with 1 for non-test commands
        let cmd = CommandRef {
            primary: PrimaryCommand::Build,
            selector: None,
        };

        let action = ext.build_action(&cmd).expect("RPC failed");
        assert!(action.is_none());
    }

    #[test]
    fn is_trusted_returns_constructor_value() {
        let trusted_ext = SubprocessExtension::new(
            "trusted".to_string(),
            "/dev/null".to_string(),
            HashSet::new(),
            true,
        );
        assert!(trusted_ext.is_trusted());

        let untrusted_ext = SubprocessExtension::new(
            "untrusted".to_string(),
            "/dev/null".to_string(),
            HashSet::new(),
            false,
        );
        assert!(!untrusted_ext.is_trusted());
    }

    #[test]
    fn binary_not_found_returns_ok_none() {
        let ext = SubprocessExtension::new(
            "ghost".to_string(),
            "/tmp/nonexistent-devflow-binary-12345".to_string(),
            HashSet::from(["test".to_string()]),
            false,
        );

        let cmd = CommandRef {
            primary: PrimaryCommand::Test,
            selector: None,
        };

        let result = ext.build_action(&cmd).expect("should not error");
        assert!(result.is_none(), "missing binary should return Ok(None)");
    }
}
