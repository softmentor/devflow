use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use anyhow::{bail, Result};

use crate::command::CommandRef;
use crate::config::DevflowConfig;
use tracing::{debug, instrument};

pub mod subprocess;

/// The action an extension wishes to execute for a given command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionAction {
    /// The executable program (e.g., "cargo", "npm").
    pub program: String,
    /// The arguments to pass to the program.
    pub args: Vec<String>,
}

/// A contract for all extensions connecting to Devflow.
pub trait Extension: std::fmt::Debug {
    /// Unique name of the extension.
    fn name(&self) -> &str;
    /// The set of command capabilities provided by this extension.
    fn capabilities(&self) -> HashSet<String>;
    /// Maps a command reference to an executable action.
    fn build_action(&self, cmd: &CommandRef) -> Option<ExecutionAction>;
}

/// A registry containing all discovered Devflow extensions.
#[derive(Debug, Default)]
pub struct ExtensionRegistry {
    extensions: HashMap<String, Box<dyn Extension>>,
}

impl ExtensionRegistry {
    /// Discovers extensions based on the provided configuration.
    ///
    /// If no extensions are explicitly configured, it attempts to load
    /// builtin extensions based on the project stack.
    pub fn discover(config: &DevflowConfig) -> Result<Self> {
        debug!(
            "discovering extensions for project: {}",
            config.project.name
        );
        let registry = Self::default();
        // Discovery logic will be delegated to the CLI / wiring phase,
        // but for backwards capability during refactor, we accept the config
        // even if we leave the registry empty.
        Ok(registry)
    }

    /// Registers a new extension into the registry.
    pub fn register(&mut self, extension: Box<dyn Extension>) {
        self.extensions
            .insert(extension.name().to_string(), extension);
    }

    /// Verifies if any registered extension can handle the given command.
    ///
    /// # Errors
    /// Returns an error if no registered extension exposes the required capability.
    #[instrument(skip(self))]
    pub fn ensure_can_run(&self, cmd: &CommandRef) -> Result<()> {
        debug!("checking capability support for: {}", cmd.canonical());
        if self.extensions.is_empty() {
            // Command planning still works without explicit extensions for early bootstrap.
            return Ok(());
        }

        let selector_key = cmd
            .selector
            .as_ref()
            .map(|selector| format!("{}:{}", cmd.primary.as_str(), selector));
        let primary_key = cmd.primary.as_str().to_string();

        let supported = self.extensions.values().any(|ext| {
            ext.capabilities().contains(&primary_key)
                || selector_key
                    .as_ref()
                    .map(|s| ext.capabilities().contains(s))
                    .unwrap_or(false)
        });

        if supported {
            return Ok(());
        }

        bail!(
            "no extension exposes capability '{}'",
            selector_key.unwrap_or(primary_key)
        )
    }

    /// Validates that all commands defined in the project targets are supported by at least one extension.
    pub fn validate_target_support(&self, cfg: &DevflowConfig) -> Result<()> {
        if self.extensions.is_empty() {
            return Ok(());
        }

        for (profile, commands) in &cfg.targets.profiles {
            for raw in commands {
                let cmd = CommandRef::from_str(raw)?;
                self.ensure_can_run(&cmd).map_err(|e| {
                    anyhow::anyhow!(
                        "unsupported command '{}' in targets profile '{}': {}",
                        raw,
                        profile,
                        e
                    )
                })?;
            }
        }

        Ok(())
    }

    /// Builds the execution arguments for a command against a specific extension.
    pub fn build_action(&self, name: &str, cmd: &CommandRef) -> Option<ExecutionAction> {
        if let Some(ext) = self.extensions.get(name) {
            ext.build_action(cmd)
        } else {
            None
        }
    }
}
