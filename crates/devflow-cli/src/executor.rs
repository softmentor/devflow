use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use devflow_core::{CommandRef, DevflowConfig, PrimaryCommand};
use tracing::{info, instrument, warn};

#[derive(Debug, Clone, Copy)]
enum Stack {
    Rust,
    Node,
    Custom,
}

/// Runs a Devflow command by dispatching it to applicable stacks.
#[instrument(skip(cfg))]
pub fn run(cfg: &DevflowConfig, command: &CommandRef) -> Result<()> {
    let stacks = resolve_stacks(cfg);
    let mut attempted = false;

    for stack in stacks {
        if !stack_is_applicable(stack) {
            info!(target: "devflow", "skip {}: manifest not found", stack_name(stack));
            continue;
        }

        let effective = with_default_selector(command);
        let Some(argv) = map_command(stack, &effective) else {
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

fn stack_is_applicable(stack: Stack) -> bool {
    match stack {
        Stack::Rust => Path::new("Cargo.toml").exists(),
        Stack::Node => Path::new("package.json").exists(),
        Stack::Custom => Path::new("justfile").exists() || Path::new("Makefile").exists(),
    }
}

fn stack_name(stack: Stack) -> &'static str {
    match stack {
        Stack::Rust => "rust",
        Stack::Node => "node",
        Stack::Custom => "custom",
    }
}

fn map_command(stack: Stack, cmd: &CommandRef) -> Option<Vec<String>> {
    match stack {
        Stack::Rust => map_rust(cmd),
        Stack::Node => map_node(cmd),
        Stack::Custom => map_custom(cmd),
    }
}

fn map_rust(cmd: &CommandRef) -> Option<Vec<String>> {
    let selector = cmd.selector.as_deref().unwrap_or("");

    match (cmd.primary, selector) {
        (PrimaryCommand::Setup, "toolchain") => Some(argv(&["rustup", "show"])),
        (PrimaryCommand::Setup, "deps") => Some(argv(&["cargo", "fetch"])),
        (PrimaryCommand::Setup, "doctor") => Some(argv(&["cargo", "--version"])),
        (PrimaryCommand::Fmt, "check") => Some(argv(&["cargo", "fmt", "--all", "--", "--check"])),
        (PrimaryCommand::Fmt, "fix") => Some(argv(&["cargo", "fmt", "--all"])),
        (PrimaryCommand::Lint, "static") => Some(vec![
            "cargo".to_string(),
            "clippy".to_string(),
            "--all-targets".to_string(),
            "--all-features".to_string(),
            "--".to_string(),
            "-D".to_string(),
            "warnings".to_string(),
        ]),
        (PrimaryCommand::Build, "debug") => Some(argv(&["cargo", "build"])),
        (PrimaryCommand::Build, "release") => Some(argv(&["cargo", "build", "--release"])),
        (PrimaryCommand::Test, "unit") => Some(argv(&["cargo", "test", "--lib", "--bins"])),
        (PrimaryCommand::Test, "integration") => Some(argv(&["cargo", "test", "--tests"])),
        (PrimaryCommand::Test, "smoke") => Some(argv(&["cargo", "test", "smoke"])),
        (PrimaryCommand::Package, "artifact") => Some(argv(&["cargo", "build", "--release"])),
        (PrimaryCommand::Release, "candidate") => Some(argv(&["cargo", "build", "--release"])),
        _ => None,
    }
}

fn map_node(cmd: &CommandRef) -> Option<Vec<String>> {
    let selector = cmd.selector.as_deref().unwrap_or("");

    match (cmd.primary, selector) {
        (PrimaryCommand::Setup, "deps") => Some(argv(&["npm", "ci"])),
        (PrimaryCommand::Setup, "doctor") => Some(argv(&["npm", "--version"])),
        (PrimaryCommand::Fmt, "check") => Some(argv(&["npm", "run", "fmt:check"])),
        (PrimaryCommand::Fmt, "fix") => Some(argv(&["npm", "run", "fmt:fix"])),
        (PrimaryCommand::Lint, "static") => Some(argv(&["npm", "run", "lint"])),
        (PrimaryCommand::Build, "debug") => Some(argv(&["npm", "run", "build"])),
        (PrimaryCommand::Build, "release") => Some(argv(&["npm", "run", "build"])),
        (PrimaryCommand::Test, "unit") => Some(argv(&["npm", "run", "test:unit"])),
        (PrimaryCommand::Test, "integration") => Some(argv(&["npm", "run", "test:integration"])),
        (PrimaryCommand::Test, "smoke") => Some(argv(&["npm", "run", "test:smoke"])),
        (PrimaryCommand::Package, "artifact") => Some(argv(&["npm", "pack", "--dry-run"])),
        _ => None,
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

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
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

    #[test]
    fn rust_mapping_exists_for_core_commands() {
        // Ensures that core Devflow commands have corresponding Cargo commands mapped for Rust.
        assert!(map_rust(&cmd(PrimaryCommand::Fmt, Some("check"))).is_some());
        assert!(map_rust(&cmd(PrimaryCommand::Lint, Some("static"))).is_some());
        assert!(map_rust(&cmd(PrimaryCommand::Build, Some("debug"))).is_some());
        assert!(map_rust(&cmd(PrimaryCommand::Test, Some("unit"))).is_some());
    }

    #[test]
    fn node_mapping_exists_for_core_commands() {
        // Ensures that core Devflow commands have corresponding npm scripts mapped for Node.
        assert!(map_node(&cmd(PrimaryCommand::Fmt, Some("check"))).is_some());
        assert!(map_node(&cmd(PrimaryCommand::Lint, Some("static"))).is_some());
        assert!(map_node(&cmd(PrimaryCommand::Build, Some("debug"))).is_some());
        assert!(map_node(&cmd(PrimaryCommand::Test, Some("unit"))).is_some());
    }
}
