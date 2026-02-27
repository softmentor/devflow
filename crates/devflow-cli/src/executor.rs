use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use devflow_core::{CommandRef, DevflowConfig, ExecutionAction, ExtensionRegistry, PrimaryCommand};
use tracing::{info, instrument, warn};

/// Runs a Devflow command by dispatching it to applicable stacks.
#[instrument(skip(cfg, registry))]
pub fn run(cfg: &DevflowConfig, registry: &ExtensionRegistry, command: &CommandRef) -> Result<()> {
    let mut attempted = false;

    for stack in &cfg.project.stack {
        if !stack_is_applicable(cfg, stack) {
            info!(target: "devflow", "skip {}: manifest not found", stack);
            continue;
        }

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
        info!(target: "devflow", "run {} on {}", effective.canonical(), stack);
        run_action(&action)
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

fn with_default_selector(command: &CommandRef) -> CommandRef {
    if command.selector.is_some() {
        return command.clone();
    }

    let selector = match command.primary {
        PrimaryCommand::Setup => "doctor",
        PrimaryCommand::Fmt => "check",
        PrimaryCommand::Lint => "static",
        PrimaryCommand::Build => "debug",
        PrimaryCommand::Test => "unit",
        PrimaryCommand::Package => "artifact",
        PrimaryCommand::Check => "pr",
        PrimaryCommand::Release => "candidate",
        PrimaryCommand::Ci => "check",
        PrimaryCommand::Init => "rust",
    };

    CommandRef {
        primary: command.primary,
        selector: Some(selector.to_string()),
    }
}

fn stack_is_applicable(cfg: &DevflowConfig, stack: &str) -> bool {
    let base = cfg.source_dir.as_deref().unwrap_or(Path::new(""));
    match stack {
        "rust" => base.join("Cargo.toml").exists(),
        "node" => base.join("package.json").exists(),
        "custom" => base.join("justfile").exists() || base.join("Makefile").exists(),
        // Subprocess extensions always apply initially; execution will fail if bad mapping
        _ => true,
    }
}

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

fn map_custom(cmd: &CommandRef) -> Option<ExecutionAction> {
    let target = cmd.canonical().replace(':', "-");

    if Path::new("justfile").exists() && command_exists("just") {
        return Some(ExecutionAction {
            program: "just".to_string(),
            args: vec![target],
        });
    }
    if Path::new("Makefile").exists() {
        return Some(ExecutionAction {
            program: "make".to_string(),
            args: vec![target],
        });
    }

    match (cmd.primary, cmd.selector.as_deref().unwrap_or("")) {
        (PrimaryCommand::Setup, "doctor") => Some(ExecutionAction {
            program: "echo".to_string(),
            args: vec!["custom stack requires justfile or Makefile targets".to_string()],
        }),
        _ => None,
    }
}

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
}
