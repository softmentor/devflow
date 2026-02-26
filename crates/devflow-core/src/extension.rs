use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use anyhow::{bail, Result};

use crate::command::CommandRef;
use crate::config::{DevflowConfig, ExtensionConfig, ExtensionSource};

#[derive(Debug, Clone)]
pub struct ExtensionDescriptor {
    pub name: String,
    pub source: ExtensionSource,
    pub version: Option<String>,
    pub api_version: u32,
    pub capabilities: HashSet<String>,
    pub required: bool,
}

#[derive(Debug, Default)]
pub struct ExtensionRegistry {
    descriptors: HashMap<String, ExtensionDescriptor>,
}

impl ExtensionRegistry {
    pub fn discover(config: &DevflowConfig) -> Result<Self> {
        let mut registry = Self::default();

        if let Some(extensions) = &config.extensions {
            for (name, entry) in extensions {
                let descriptor = descriptor_from_config(name, entry)?;
                registry.descriptors.insert(name.clone(), descriptor);
            }
        }

        Ok(registry)
    }

    pub fn ensure_can_run(&self, cmd: &CommandRef) -> Result<()> {
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
}
