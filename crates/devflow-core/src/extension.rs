use std::collections::{HashMap, HashSet};

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
}

fn descriptor_from_config(name: &str, entry: &ExtensionConfig) -> Result<ExtensionDescriptor> {
    let mut capabilities = entry.capabilities.iter().cloned().collect::<HashSet<_>>();

    if capabilities.is_empty() {
        // If no capabilities are declared, fall back to the extension key name.
        capabilities.insert(name.to_string());
    }

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
