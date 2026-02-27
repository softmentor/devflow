use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use anyhow::{bail, Result};

use crate::command::CommandRef;
use crate::config::{DevflowConfig, ExtensionConfig, ExtensionSource};
use tracing::{debug, instrument};

/// Describes a Devflow extension and its capabilities.
#[derive(Debug, Clone)]
pub struct ExtensionDescriptor {
    /// Unique name of the extension.
    pub name: String,
    /// Where the extension is sourced from.
    pub source: ExtensionSource,
    /// Extension version.
    pub version: Option<String>,
    /// The API version this extension implements.
    pub api_version: u32,
    /// The set of command capabilities provided by this extension.
    pub capabilities: HashSet<String>,
    /// Whether this extension is required for the project.
    pub required: bool,
}

/// A registry containing all discovered Devflow extensions.
#[derive(Debug, Default)]
pub struct ExtensionRegistry {
    descriptors: HashMap<String, ExtensionDescriptor>,
}

impl ExtensionRegistry {
    /// Discovers extensions based on the provided configuration.
    ///
    /// If no extensions are explicitly configured, it attempts to load
    /// builtin extensions based on the project stack.
    #[instrument(skip(config))]
    pub fn discover(config: &DevflowConfig) -> Result<Self> {
        debug!(
            "discovering extensions for project: {}",
            config.project.name
        );
        let mut registry = Self::default();

        match &config.extensions {
            Some(extensions) => {
                for (name, entry) in extensions {
                    let descriptor = descriptor_from_config(name, entry)?;
                    registry.descriptors.insert(name.clone(), descriptor);
                }
            }
            None => {
                for stack in &config.project.stack {
                    if stack == "custom" {
                        continue;
                    }
                    let descriptor = builtin_descriptor_for_stack(stack)?;
                    registry.descriptors.insert(stack.clone(), descriptor);
                }
            }
        }

        Ok(registry)
    }

    /// Verifies if any registered extension can handle the given command.
    ///
    /// # Errors
    /// Returns an error if no registered extension exposes the required capability.
    #[instrument(skip(self))]
    pub fn ensure_can_run(&self, cmd: &CommandRef) -> Result<()> {
        debug!("checking capability support for: {}", cmd.canonical());
        if self.descriptors.is_empty() {
            // Command planning still works without explicit extensions for early bootstrap.
            return Ok(());
        }

        let selector_key = cmd
            .selector
            .as_ref()
            .map(|selector| format!("{}:{}", cmd.primary.as_str(), selector));
        let primary_key = cmd.primary.as_str().to_string();

        let supported = self.descriptors.values().any(|ext| {
            ext.capabilities.contains(&primary_key)
                || selector_key
                    .as_ref()
                    .map(|s| ext.capabilities.contains(s))
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
        if self.descriptors.is_empty() {
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
    pub fn build_command(&self, name: &str, cmd: &CommandRef) -> Option<Vec<String>> {
        if !self.descriptors.contains_key(name) {
            return None;
        }

        let primary = cmd.primary.as_str();
        let selector = cmd.selector.as_deref().unwrap_or("");

        match name {
            "rust" => devflow_ext_rust::build_command(primary, selector),
            "node" => devflow_ext_node::build_command(primary, selector),
            _ => None,
        }
    }
}

fn builtin_descriptor_for_stack(stack: &str) -> Result<ExtensionDescriptor> {
    let capabilities = builtin_capabilities(stack).ok_or_else(|| {
        anyhow::anyhow!(
            "no builtin extension available for stack '{}' (supported: rust,node)",
            stack
        )
    })?;

    Ok(ExtensionDescriptor {
        name: stack.to_string(),
        source: ExtensionSource::Builtin,
        version: None,
        api_version: 1,
        capabilities,
        required: true,
    })
}

fn descriptor_from_config(name: &str, entry: &ExtensionConfig) -> Result<ExtensionDescriptor> {
    let mut capabilities = entry.capabilities.iter().cloned().collect::<HashSet<_>>();

    if let ExtensionSource::Path = &entry.source {
        let path = entry.path.as_ref().ok_or_else(|| {
            anyhow::anyhow!("extension '{}' source=path requires a 'path' value", name)
        })?;
        if !path.exists() {
            bail!(
                "extension '{}' path does not exist: {}",
                name,
                path.display()
            );
        }
    }

    let api_version = entry.api_version.unwrap_or(1);
    if api_version != 1 {
        bail!(
            "extension '{}' has unsupported api_version={} (expected 1)",
            name,
            api_version
        );
    }

    if capabilities.is_empty() {
        capabilities = match entry.source {
            ExtensionSource::Builtin => builtin_capabilities(name).unwrap_or_default(),
            ExtensionSource::Path => HashSet::new(),
        };
    }

    if capabilities.is_empty() {
        bail!(
            "extension '{}' has no capabilities; set capabilities in config or use a known builtin",
            name
        );
    }

    let descriptor = ExtensionDescriptor {
        name: name.to_string(),
        source: entry.source.clone(),
        version: entry.version.clone(),
        api_version,
        capabilities,
        required: entry.required,
    };

    Ok(descriptor)
}

fn builtin_capabilities(name: &str) -> Option<HashSet<String>> {
    let values: &[&str] = match name {
        "rust" => devflow_ext_rust::default_capabilities(),
        "node" => devflow_ext_node::default_capabilities(),
        _ => return None,
    };
    Some(values.iter().map(|item| (*item).to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(toml_text: &str) -> DevflowConfig {
        toml::from_str(toml_text).expect("fixture config should parse")
    }

    #[test]
    fn validates_supported_target_commands() {
        // Verifies that the registry correctly identifies and validates
        // commands that are supported by registered extensions.
        let cfg = fixture(
            r#"
            [project]
            name = "demo"
            stack = ["rust"]

            [targets]
            pr = ["fmt:check", "test:unit"]

            [extensions.rust]
            source = "builtin"
            required = true
            "#,
        );

        let registry = ExtensionRegistry::discover(&cfg).expect("discover should pass");
        registry
            .validate_target_support(&cfg)
            .expect("supported commands should validate");
    }

    #[test]
    fn rejects_unsupported_target_commands() {
        // Ensures that the registry fails validation if a target command
        // is not supported by any registered extension.
        let cfg = fixture(
            r#"
            [project]
            name = "demo"
            stack = ["rust"]

            [targets]
            pr = ["test:unknown_selector"]

            [extensions.rust]
            source = "builtin"
            required = true
            "#,
        );

        let registry = ExtensionRegistry::discover(&cfg).expect("discover should pass");
        let err = registry
            .validate_target_support(&cfg)
            .expect_err("unsupported command must fail");
        assert!(err.to_string().contains("unsupported command"));
        assert!(err.to_string().contains("pr"));
    }

    #[test]
    fn auto_loads_builtin_extensions_when_config_not_present() {
        // Verifies that Devflow automatically discovers builtin extensions
        // based on the project stack if no explicit extensions are defined.
        let cfg = fixture(
            r#"
            [project]
            name = "demo"
            stack = ["rust"]

            [targets]
            pr = ["fmt:check", "test:unit"]
            "#,
        );

        let registry = ExtensionRegistry::discover(&cfg).expect("discover should pass");
        registry
            .validate_target_support(&cfg)
            .expect("builtin extension should validate targets");
    }
}
