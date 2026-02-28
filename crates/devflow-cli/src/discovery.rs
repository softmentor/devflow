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

#[cfg(test)]
mod tests {
    use super::*;
    use devflow_core::command::CommandRef;
    use devflow_core::config::{DevflowConfig, ProjectConfig, RuntimeConfig};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::str::FromStr;
    use tempfile::tempdir;

    fn create_mock_binary(dir_path: &std::path::Path, stack: &str, capabilities_json: &str) {
        let binary_name = format!("devflow-ext-{}", stack);
        let path = dir_path.join(binary_name);

        let script = format!(
            r#"#!/usr/bin/env sh
if [ "$1" = "--discover" ]; then
    echo '{}'
    exit 0
fi
exit 1
"#,
            capabilities_json
        );

        fs::write(&path, script).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }

    #[test]
    fn discover_subprocess_extensions_success() {
        let dir = tempdir().unwrap();

        // Put the temp directory in the PATH for this test so Command::new finds our script
        let old_path = std::env::var_os("PATH").unwrap_or_default();
        let mut new_path = dir.path().to_path_buf().into_os_string();
        new_path.push(":");
        new_path.push(&old_path);
        std::env::set_var("PATH", &new_path);

        create_mock_binary(dir.path(), "python", r#"["test", "fmt"]"#);

        let cfg = DevflowConfig {
            project: ProjectConfig {
                name: "test-proj".to_string(),
                stack: vec!["python".to_string()],
            },
            runtime: RuntimeConfig {
                profile: devflow_core::runtime::RuntimeProfile::default(),
            },
            cache: Default::default(),
            container: Default::default(),
            extensions: Default::default(),
            targets: Default::default(),
            source_dir: None,
        };

        let mut registry = ExtensionRegistry::default();
        let result = discover_subprocess_extensions(&cfg, &mut registry);

        // Reset PATH immediately
        std::env::set_var("PATH", old_path);

        assert!(result.is_ok());

        // Verify the extension was registered
        let cmd = CommandRef::from_str("test").unwrap();
        assert!(registry.ensure_can_run(&cmd).is_ok());

        let cmd_fmt = CommandRef::from_str("fmt").unwrap();
        assert!(registry.ensure_can_run(&cmd_fmt).is_ok());
    }

    #[test]
    fn discover_subprocess_extensions_ignores_builtin() {
        let cfg = DevflowConfig {
            project: ProjectConfig {
                name: "test-proj".to_string(),
                stack: vec!["rust".to_string(), "node".to_string(), "custom".to_string()],
            },
            runtime: RuntimeConfig {
                profile: devflow_core::runtime::RuntimeProfile::default(),
            },
            cache: Default::default(),
            container: Default::default(),
            extensions: Default::default(),
            targets: Default::default(),
            source_dir: None,
        };

        let mut registry = ExtensionRegistry::default();
        let result = discover_subprocess_extensions(&cfg, &mut registry);
        assert!(result.is_ok());

        // Ensure no extensions were actually added for builtins
        let cmd = CommandRef::from_str("test").unwrap();
        // Since the registry is empty, `ensure_can_run` might return Ok(()) to allow bootstrap,
        // but `build_action` for a specific named builtin extension like "rust" or an unregistered one will be None
        assert!(registry.build_action("rust", &cmd).is_none());
    }
}
