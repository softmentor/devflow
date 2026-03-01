//! Command execution engine and container orchestration.
//!
//! This module handles the dispatch of Devflow commands to their respective
//! extensions. It also provides the "container proxy" implementation that
//! wraps host commands in Docker/Podman `run` calls with transparent volume mounting.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use devflow_core::{
    config::ContainerEngine, runtime::RuntimeProfile, CommandRef, DevflowConfig, ExecutionAction,
    ExtensionRegistry, PrimaryCommand,
};
use tracing::{info, instrument, warn};

/// Default image used for containerized execution if none specified.
const DEFAULT_CI_IMAGE: &str = "ghcr.io/softmentor/devflow-ci:latest";
/// Default host directory for the Devflow cache.
const DEFAULT_CACHE_ROOT: &str = ".cache/devflow";
/// The internal container path where the project is mounted.
const CONTAINER_WORKSPACE: &str = "/workspace";
/// The internal container path where the host `dwf` binary is mapped.
const CONTAINER_DWF_BIN: &str = "/usr/local/bin/dwf";

/// Runs a Devflow command by dispatching it to applicable stacks.
#[instrument(skip(cfg, registry), fields(command = %command))]
pub fn run(cfg: &DevflowConfig, registry: &ExtensionRegistry, command: &CommandRef) -> Result<()> {
    let mut attempted = false;

    let mut requested_stacks = Vec::new();
    for stack in &cfg.project.stack {
        if stack_is_applicable(cfg, stack) {
            requested_stacks.push(stack.clone());
        } else {
            info!(target: "devflow", "skip {}: manifest not found", stack);
        }
    }

    if let Some(extensions) = &cfg.extensions {
        for ext_name in extensions.keys() {
            if !requested_stacks.contains(ext_name) {
                // Explicitly declared subprocess extensions assume implicit applicability
                requested_stacks.push(ext_name.clone());
            }
        }
    }

    for stack in &requested_stacks {
        let effective = with_default_selector(command);
        let Some(action) = map_command(stack, &effective, registry) else {
            info!(target: "devflow",
                "skip {}: unsupported command {}",
                stack,
                effective.canonical()
            );
            continue;
        };

        attempted = true;

        // When IS_CONTAINER=true (e.g., inside GHA native container: job),
        // skip the docker-run proxy even if profile is "container".
        // This enables GHA native container jobs to run dwf commands directly.
        let is_already_in_container = std::env::var("IS_CONTAINER")
            .map(|v| v == "true")
            .unwrap_or(false);

        let final_action =
            if cfg.runtime.profile == RuntimeProfile::Container && !is_already_in_container {
                build_container_proxy(cfg, registry, &action)?
            } else {
                action
            };

        info!(target: "devflow", "run {} on {}", effective, stack);
        run_action(&final_action)
            .with_context(|| format!("{} failed for {}", effective.canonical(), stack))?;
    }

    if !attempted {
        bail!(
            "command '{}' did not match any runnable stack",
            command.canonical()
        );
    }

    Ok(())
}

/// Normalizes a command by applying default selectors if missing.
fn with_default_selector(command: &CommandRef) -> CommandRef {
    if command.selector.is_some() {
        return command.clone();
    }

    CommandRef {
        primary: command.primary,
        selector: Some(command.primary.default_selector().to_string()),
    }
}

/// Checks if a stack-specific manifest (e.g., Cargo.toml) exists in the source directory.
fn stack_is_applicable(cfg: &DevflowConfig, stack: &str) -> bool {
    let base = cfg.source_dir.as_deref().unwrap_or(Path::new(""));
    devflow_core::project::stack_is_applicable(base, stack)
}

/// Maps a logical Devflow command to a concrete execution action for a given stack.
fn map_command(
    stack: &str,
    cmd: &CommandRef,
    registry: &ExtensionRegistry,
) -> Option<ExecutionAction> {
    match stack {
        "custom" => map_custom(cmd),
        _ => registry.build_action(stack, cmd),
    }
}

/// Fallback logic for projects using `Makefile` or `justfile` without a specific Devflow extension.
fn map_custom(cmd: &CommandRef) -> Option<ExecutionAction> {
    let target = cmd.canonical().replace(':', "-");

    if Path::new("justfile").exists() && command_exists("just") {
        return Some(ExecutionAction {
            program: "just".to_string(),
            args: vec![target],
            env: std::collections::HashMap::new(),
        });
    }
    if Path::new("Makefile").exists() {
        return Some(ExecutionAction {
            program: "make".to_string(),
            args: vec![target],
            env: std::collections::HashMap::new(),
        });
    }

    match (cmd.primary, cmd.selector.as_deref().unwrap_or("")) {
        (PrimaryCommand::Setup, "doctor") => Some(ExecutionAction {
            program: "echo".to_string(),
            args: vec!["custom stack requires justfile or Makefile targets".to_string()],
            env: std::collections::HashMap::new(),
        }),
        _ => None,
    }
}

/// Executes a process on the host system.
fn run_action(action: &ExecutionAction) -> Result<()> {
    let status = Command::new(&action.program)
        .args(&action.args)
        .status()
        .with_context(|| {
            format!(
                "failed to start command '{} {}'",
                action.program,
                action.args.join(" ")
            )
        })?;

    if !status.success() {
        bail!(
            "command failed with status {}: {} {}",
            status,
            action.program,
            action.args.join(" ")
        );
    }

    Ok(())
}

/// Transforms a host execution action into a containerized proxy action.
///
/// This involves:
/// 1. Detecting an available container engine (Docker/Podman).
/// 2. Resolving the appropriate container image.
/// 3. Injecting the host `dwf` binary into the container to ensure version parity.
/// 4. Mounting the workspace and any extension-defined cache volumes.
fn build_container_proxy(
    cfg: &DevflowConfig,
    registry: &ExtensionRegistry,
    action: &ExecutionAction,
) -> Result<ExecutionAction> {
    let container_config = cfg.container.as_ref();
    let engine_cfg = container_config.map(|c| c.engine).unwrap_or_default();

    let engine_cmd = resolve_engine(engine_cfg)?;

    let image = container_config
        .and_then(|c| c.image.clone())
        .unwrap_or_else(|| DEFAULT_CI_IMAGE.to_string());

    let dwf_cache_root = std::env::var("DWF_CACHE_ROOT")
        .ok()
        .or_else(|| cfg.cache.as_ref().and_then(|c| c.root.clone()))
        .unwrap_or_else(|| DEFAULT_CACHE_ROOT.to_string());

    let cwd = std::env::current_dir()?;
    let cwd_str = cwd.to_string_lossy();

    // Version parity safety: we map the host's actively executing `dwf` binary
    // into the container so that even if the container image is old, it always
    // uses the exact same Devflow logic as the invoker.
    let host_dwf_path = std::env::current_exe()?;
    let host_dwf_str = host_dwf_path.to_string_lossy();

    let mut args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        format!("{}:{}", cwd_str, CONTAINER_WORKSPACE),
        "-v".to_string(),
        format!("{}:{}:ro", host_dwf_str, CONTAINER_DWF_BIN),
        "-w".to_string(),
        CONTAINER_WORKSPACE.to_string(),
    ];

    // Cache redirection: extensions define relative paths (e.g. ".cargo") which
    // we anchor to the unified `DWF_CACHE_ROOT` on the host.
    let abs_cache_root = resolve_cache_root(cfg, &dwf_cache_root);
    let mounts = registry.all_cache_mounts();

    for mount in mounts {
        if let Some((host_rel, container_abs)) = parse_mount(&mount) {
            let host_abs = abs_cache_root.join(host_rel);

            if let Err(e) = std::fs::create_dir_all(&host_abs) {
                warn!(
                    "failed to create cache directory {}: {}",
                    host_abs.display(),
                    e
                );
            }

            args.push("-v".to_string());
            args.push(format!("{}:{}", host_abs.display(), container_abs));
        } else {
            warn!("invalid cache mount format from extension: {}", mount);
        }
    }

    for (key, value) in &action.env {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, value));
    }

    args.push(image);
    args.push(action.program.clone());
    args.extend(action.args.clone());

    Ok(ExecutionAction {
        program: engine_cmd,
        args,
        env: action.env.clone(),
    })
}

fn resolve_engine(engine_cfg: ContainerEngine) -> Result<String> {
    let cmd = match engine_cfg {
        ContainerEngine::Docker => "docker",
        ContainerEngine::Podman => "podman",
        ContainerEngine::Auto => {
            if is_engine_healthy("podman") {
                "podman"
            } else if is_engine_healthy("docker") {
                "docker"
            } else if command_exists("podman") {
                "podman"
            } else if command_exists("docker") {
                "docker"
            } else {
                bail!("no container engine (docker or podman) found on PATH");
            }
        }
    };

    if !command_exists(cmd) {
        bail!("required container engine '{cmd}' is not installed or not on PATH");
    }

    info!(target: "devflow", "using container engine: {}", cmd);
    Ok(cmd.to_string())
}

/// Checks if an engine is not only installed but also has a responsive daemon.
fn is_engine_healthy(name: &str) -> bool {
    if !command_exists(name) {
        return false;
    }
    // 'info' usually requires a working daemon link
    Command::new(name)
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn resolve_cache_root(cfg: &DevflowConfig, root: &str) -> PathBuf {
    let path = PathBuf::from(root);
    if path.is_absolute() {
        return path;
    }

    let source_dir = cfg.source_dir.as_deref().unwrap_or_else(|| Path::new("."));
    let abs_source = if source_dir.is_absolute() {
        source_dir.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(source_dir)
    };

    abs_source.join(root)
}

fn parse_mount(mount: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = mount.split(':').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

/// Checks if a command exists on the host system.
fn command_exists(name: &str) -> bool {
    Command::new(name).arg("--version").status().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use devflow_core::PrimaryCommand;

    fn cmd(primary: PrimaryCommand, selector: Option<&str>) -> CommandRef {
        CommandRef {
            primary,
            selector: selector.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn default_selector_is_applied() {
        // Verifies that a primary command without a selector gets a sensible default.
        let out = with_default_selector(&cmd(PrimaryCommand::Fmt, None));
        assert_eq!(out.canonical(), "fmt:check");
    }

    #[test]
    fn explicit_selector_is_preserved() {
        // Verifies that if a selector is already present, it is not overwritten by defaults.
        let out = with_default_selector(&cmd(PrimaryCommand::Test, Some("integration")));
        assert_eq!(out.canonical(), "test:integration");
    }

    #[test]
    fn unit_test_map_custom_translates_selectors() {
        // map_custom depends on filesystem state (justfile/Makefile).
        // Since those don't exist in the standard test dir by default, it usually falls back.
        // We will test the fallback behavior here.
        let out = map_custom(&cmd(PrimaryCommand::Setup, Some("doctor"))).unwrap();
        assert_eq!(out.program, "echo");
        assert!(out.args[0].contains("custom stack requires"));

        // Unhandled commands return None in the default fallback
        assert!(map_custom(&cmd(PrimaryCommand::Build, Some("debug"))).is_none());
    }

    #[test]
    fn integration_test_run_action_success() {
        let action = ExecutionAction {
            program: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            env: std::collections::HashMap::new(),
        };
        // Should succeed without error
        assert!(run_action(&action).is_ok());
    }

    #[test]
    fn integration_test_run_action_failure() {
        let action = ExecutionAction {
            program: "false".to_string(), // Typical unix command that always fails
            args: vec![],
            env: std::collections::HashMap::new(),
        };
        let result = run_action(&action);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("command failed with status"));
    }

    #[test]
    fn integration_test_run_action_invalid_program() {
        let action = ExecutionAction {
            program: "this-program-definitely-does-not-exist-123".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
        };
        let result = run_action(&action);
        assert!(result.is_err());
    }
}
