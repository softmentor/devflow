use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use devflow_core::{CommandRef, DevflowConfig, ExtensionRegistry, PrimaryCommand};
use tracing::{info, instrument, warn};

#[derive(Debug, Clone, Copy)]
enum Stack {
    Rust,
    Node,
    Custom,
}

/// Runs a Devflow command by dispatching it to applicable stacks.
#[instrument(skip(cfg, registry))]
pub fn run(cfg: &DevflowConfig, registry: &ExtensionRegistry, command: &CommandRef) -> Result<()> {
    let stacks = resolve_stacks(cfg);
    let mut attempted = false;

    for stack in stacks {
        if !stack_is_applicable(cfg, stack) {
            info!(target: "devflow", "skip {}: manifest not found", stack_name(stack));
            continue;
        }

        let effective = with_default_selector(command);
        let Some(argv) = map_command(stack, &effective, registry) else {
            info!(target: "devflow",
                "skip {}: unsupported command {}",
                stack_name(stack),
                effective.canonical()
            );
            continue;
        };

        attempted = true;
        info!(target: "devflow", "run {} on {}", effective.canonical(), stack_name(stack));
        run_argv(&argv).with_context(|| {
            format!("{} failed for {}", effective.canonical(), stack_name(stack))
        })?;
    }

    if !attempted {
        bail!(
            "command '{}' did not match any runnable stack",
            command.canonical()
        );
    }

    Ok(())
}

fn resolve_stacks(cfg: &DevflowConfig) -> Vec<Stack> {
    cfg.project
        .stack
        .iter()
        .filter_map(|value| match value.as_str() {
            "rust" => Some(Stack::Rust),
            "node" => Some(Stack::Node),
            "custom" => Some(Stack::Custom),
            _ => {
                warn!("unknown stack '{}' ignored", value);
                None
            }
        })
        .collect()
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

fn stack_is_applicable(cfg: &DevflowConfig, stack: Stack) -> bool {
    let base = cfg.source_dir.as_deref().unwrap_or(Path::new(""));
    match stack {
        Stack::Rust => base.join("Cargo.toml").exists(),
        Stack::Node => base.join("package.json").exists(),
        Stack::Custom => base.join("justfile").exists() || base.join("Makefile").exists(),
    }
}

fn stack_name(stack: Stack) -> &'static str {
    match stack {
        Stack::Rust => "rust",
        Stack::Node => "node",
        Stack::Custom => "custom",
    }
}

fn map_command(
    stack: Stack,
    cmd: &CommandRef,
    registry: &ExtensionRegistry,
) -> Option<Vec<String>> {
    match stack {
        Stack::Custom => map_custom(cmd),
        _ => registry.build_command(stack_name(stack), cmd),
    }
}

fn map_custom(cmd: &CommandRef) -> Option<Vec<String>> {
    let target = cmd.canonical().replace(':', "-");

    if Path::new("justfile").exists() && command_exists("just") {
        return Some(vec!["just".to_string(), target]);
    }
    if Path::new("Makefile").exists() {
        return Some(vec!["make".to_string(), target]);
    }

    match (cmd.primary, cmd.selector.as_deref().unwrap_or("")) {
        (PrimaryCommand::Setup, "doctor") => Some(vec![
            "echo".to_string(),
            "custom stack requires justfile or Makefile targets".to_string(),
        ]),
        _ => None,
    }
}

fn run_argv(argv: &[String]) -> Result<()> {
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("empty command argv"))?;

    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("failed to start command '{} {}'", program, args.join(" ")))?;

    if !status.success() {
        bail!(
            "command failed with status {}: {} {}",
            status,
            program,
            args.join(" ")
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
