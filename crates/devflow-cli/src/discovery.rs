use std::collections::HashSet;
use std::process::Command;

use anyhow::Result;
use tracing::{debug, warn};

use devflow_core::extension::subprocess::SubprocessExtension;
use devflow_core::{DevflowConfig, ExtensionRegistry};

pub fn discover_subprocess_extensions(
    cfg: &DevflowConfig,
    registry: &mut ExtensionRegistry,
) -> Result<()> {
    for stack in &cfg.project.stack {
        // Skip built-in extensions we already registered explicitly and the custom stack logic
        if stack == "rust" || stack == "node" || stack == "custom" {
            continue;
        }

        let binary_name = format!("devflow-ext-{}", stack);

        debug!("probing for subprocess extension: {}", binary_name);

        let output = match Command::new(&binary_name).arg("--discover").output() {
            Ok(out) => out,
            Err(e) => {
                debug!("failed to find or execute '{}': {}", binary_name, e);
                continue;
            }
        };

        if !output.status.success() {
            warn!(
                "{} --discover failed with status {}",
                binary_name, output.status
            );
            continue;
        }

        let capabilities: HashSet<String> = match serde_json::from_slice(&output.stdout) {
            Ok(caps) => caps,
            Err(e) => {
                warn!("failed to parse capabilities from {}: {}", binary_name, e);
                continue;
            }
        };

        debug!(
            "discovered subprocess extension '{}' with capabilities: {:?}",
            stack, capabilities
        );

        let ext = SubprocessExtension::new(stack.clone(), binary_name, capabilities);
        registry.register(Box::new(ext));
    }

    Ok(())
}
