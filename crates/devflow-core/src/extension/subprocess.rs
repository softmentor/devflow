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
